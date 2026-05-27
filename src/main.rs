mod app;
mod controls;
mod gui;
mod helpers;
mod primitives;
mod renderer;
mod scene;

use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

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
