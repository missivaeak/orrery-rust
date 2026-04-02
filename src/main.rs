mod constants;
mod math;
mod primitives;
mod renderer;
mod state;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

use crate::{renderer::Renderer, state::State};

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // let primitive_type = {
    //     if args.len() > 1 {
    //         &args[1]
    //     } else {
    //         "vertex_buffer"
    //     }
    // };

    let event_loop = EventLoop::new().unwrap();

    let mut app = App::default();

    // event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut app).unwrap();
}

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("Initialising...");

        let size = LogicalSize::new(512.0, 512.0);
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

        let renderer = pollster::block_on(renderer::Renderer::new(window.clone()));
        println!("WGPU initialised");

        let state = State::new(&renderer.device, size.width / size.height);
        println!("State initialised");

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.state = Some(state);

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
                    && let Some(state) = &mut self.state
                {
                    state.update(&renderer.device);
                    let (vertex_uniform, fragment_uniform) = state.get_global_uniforms();
                    let objects = state.get_objects();
                    match renderer.render(objects, vertex_uniform, fragment_uniform) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }
            _ => (),
        }
    }
}
