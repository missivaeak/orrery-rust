use std::{fs, sync::Arc, time::Instant};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{CursorGrabMode, Window, WindowId},
};

use crate::{
    controls::{Controls, InputEventResult},
    gui::Gui,
    helpers::constants::ASPECT_RATIO,
    renderer::Renderer,
    scene::Scene,
};

struct FrameCount<const N: usize> {
    index: usize,
    ready: bool,
    frame_ms: [f32; N],
}

impl<const N: usize> FrameCount<N> {
    fn len(&self) -> usize {
        N
    }
    fn advance(&mut self) {
        if self.index + 1 == N {
            self.ready = true;
        }
        self.index = (self.index + 1) % N;
    }
    fn set_frame(&mut self, frame_seconds: f32) {
        self.frame_ms[self.index] = frame_seconds;
    }
}

impl<const N: usize> Default for FrameCount<N> {
    fn default() -> Self {
        Self {
            index: 0,
            ready: false,
            frame_ms: [0.0166; N],
        }
    }
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    scene: Option<Scene>,
    controls: Option<Controls>,
    gui: Option<Gui>,
    frame_count: FrameCount<600>,
    initial_instant: Option<Instant>,
    last_instant: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Initialising...");

        let size = LogicalSize::new(512.0 * ASPECT_RATIO, 512.0);
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("rust-renderer")
                        .with_inner_size(size),
                )
                .expect("Failed to create window"),
        );
        println!("Window initialised");

        let renderer = pollster::block_on(Renderer::new(
            window.clone(),
            Box::new(event_loop.owned_display_handle()),
        ));
        println!("WGPU initialised");

        let controls = Controls::new();
        println!("Controls initialised");

        let scene = Scene::new(&renderer.device, &controls, size);
        println!("Scene initialised");

        let gui = Gui::new(
            &renderer.device,
            &window,
            size,
            renderer.get_texture_format(),
        );
        println!("GUI initialised");

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.scene = Some(scene);
        self.controls = Some(controls);
        self.gui = Some(gui);
        self.initial_instant = Some(Instant::now());
        self.last_instant = Some(Instant::now());

        println!("Initialisation completed")
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event
            && let Some(controls) = &mut self.controls
        {
            controls.handle_mouse_move(delta);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(last_instant) = self.last_instant
                    && let frame_count = &mut self.frame_count
                    && let Some(gui) = &mut self.gui
                {
                    let time_elapsed_frame =
                        Instant::now().duration_since(last_instant).as_secs_f32();
                    frame_count.advance();
                    frame_count.set_frame(time_elapsed_frame * 1000.0);
                    let average_frame_ms =
                        frame_count.frame_ms.iter().sum::<f32>() / frame_count.len() as f32;
                    if frame_count.ready {
                        gui.set_frame_ms(average_frame_ms);
                        frame_count.ready = false;
                    }
                }

                self.last_instant = Some(Instant::now());

                if let Some(gui) = &mut self.gui
                    && let Some(controls) = &mut self.controls
                {
                    gui.set_camera_position(controls.camera_position);
                }

                if let Some(renderer) = &mut self.renderer
                    && let Some(scene) = &mut self.scene
                    && let Some(controls) = &mut self.controls
                    && let Some(gui) = &mut self.gui
                    && let Some(window) = &self.window
                    && let Some(initial_instant) = self.initial_instant
                {
                    let time_elapsed_total = initial_instant.elapsed().as_secs_f32();
                    // println!("t: {:?}", time_elapsed_frame);
                    controls.update();
                    scene.update(
                        &renderer.device,
                        controls.get_view_mat().into(),
                        time_elapsed_total,
                    );
                    let (vertex_uniform, fragment_uniform) = scene.get_global_uniforms();
                    let objects = scene.get_objects();
                    gui.begin_frame(window);

                    renderer.render(
                        objects,
                        vertex_uniform,
                        fragment_uniform,
                        |device, queue, encoder, view| {
                            gui.end_frame_and_draw(device, queue, encoder, window, view)
                        },
                    );
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(controls) = &mut self.controls {
                    match controls.handle_key_input(event) {
                        InputEventResult::RequestClose => {
                            println!("Close requested by key input");
                            let _ = fs::write(".restart-trigger", "restart");
                            event_loop.exit()
                        }
                        InputEventResult::Ok => (),
                        _ => (),
                    }
                }
            }

            WindowEvent::MouseInput { button, state, .. } => {
                if let Some(controls) = &mut self.controls
                    && let Some(window) = &mut self.window
                    && state == ElementState::Pressed
                {
                    match controls.handle_mouse_input(button) {
                        InputEventResult::RequestLockCursor => {
                            window.set_cursor_visible(false);
                            window
                                .set_cursor_grab(CursorGrabMode::Locked)
                                .expect("Failed to set cursor lock");
                        }
                        InputEventResult::RequestUnlockCursor => {
                            window.set_cursor_visible(true);
                            window
                                .set_cursor_grab(CursorGrabMode::None)
                                .expect("Failed to unset cursor lock");
                        }
                        _ => (),
                    }
                }
            }

            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer
                    && let Some(gui) = &mut self.gui
                {
                    renderer.resize(new_size);
                    gui.resize(new_size);
                }
            }
            _ => (),
        }
    }
}
