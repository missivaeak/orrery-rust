use std::{fs, sync::Arc, time::Instant};

use egui::frame;
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{
    controls::{Controls, InputEventResult},
    gui::Gui,
    helpers::constants::ASPECT_RATIO,
    renderer::Renderer,
    scene::Scene,
};

struct FrameCount {
    last_index: usize,
    frame_ms: [u128; 60],
}

impl Default for FrameCount {
    fn default() -> Self {
        Self {
            last_index: 0,
            frame_ms: [0; 60],
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
    frame_count: FrameCount,
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

        let scene = Scene::new(&renderer.device, size);
        println!("Scene initialised");

        let controls = Controls::new();
        println!("Controls initialised");

        let gui = Gui::new(&renderer.device, &window, size);
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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer
                    && let Some(scene) = &mut self.scene
                    && let Some(controls) = &mut self.controls
                    && let Some(gui) = &mut self.gui
                    && let Some(window) = &self.window
                    && let Some(initial_instant) = self.initial_instant
                    && let Some(last_instant) = &self.last_instant
                    && let frame_count = &mut self.frame_count
                {
                    let time_elapsed_total = initial_instant.elapsed().as_secs_f32();
                    let time_elapsed_frame = last_instant.elapsed().as_millis();
                    let i = frame_count.last_index;
                    frame_count.frame_ms[i] = time_elapsed_frame;
                    frame_count.last_index = (i + 1) % 60;
                    controls.update();
                    scene.update(
                        &renderer.device,
                        controls.get_view_mat().into(),
                        time_elapsed_total,
                    );
                    let (vertex_uniform, fragment_uniform) = scene.get_global_uniforms();
                    let objects = scene.get_objects();
                    let average_frame_ms = frame_count.frame_ms.iter().sum::<u128>() as f32 / 60.0;
                    gui.begin_frame(window, average_frame_ms);
                    println!("{:?}", average_frame_ms);

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

                // println!("{:?}", self.frame_count.frame_ms);

                self.last_instant = Some(Instant::now());
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
