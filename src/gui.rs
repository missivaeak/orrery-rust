use egui::{Context, ViewportId};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use winit::{
    dpi::LogicalSize,
    event::WindowEvent,
    window::{Theme, Window},
};

pub struct Gui {
    state: Option<State>,
    renderer: Option<Renderer>,
    frame_started: bool,
    pub screen_descriptor: ScreenDescriptor,
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            state: None,
            renderer: None,
            frame_started: false,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [512, 512],
                pixels_per_point: 1.0,
            },
        }
    }
}

impl Gui {
    pub fn new(
        device: &Device,
        window: &Window,
        output_colour_format: TextureFormat,
        size: LogicalSize<f32>,
    ) -> Self {
        let context = Context::default();
        let state = State::new(
            context,
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            Some(Theme::Light),
            Some(2 * 1024),
        );
        let renderer = Renderer::new(device, output_colour_format, RendererOptions::default());
        Self {
            renderer,
            state,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [size.width as u32, size.height as u32],
                pixels_per_point: 1.0,
            },
            frame_started: false,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn ppp(&mut self, v: f32) {
        self.state.egui_ctx().set_pixels_per_point(v);
    }

    pub fn begin_frame(&mut self, window: &Window) {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        if !self.frame_started {
            panic!("begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(screen_descriptor.pixels_per_point);

        let full_output = self.state.egui_ctx().end_pass();

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, encoder, &tris, &screen_descriptor);
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: window_surface_view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("egui main render pass"),
            occlusion_query_set: None,
            multiview_mask: None,
        });

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }
}
