use egui::{
    Align, Align2, Area, Color32, Context, CornerRadius, Frame, Layout, Margin, Shadow, ViewportId,
};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    window::{Theme, Window},
};

pub struct Gui {
    state: State,
    renderer: Renderer,
    frame_started: bool,
    pub screen_descriptor: ScreenDescriptor,
}

impl Gui {
    pub fn new(device: &Device, window: &Window, size: LogicalSize<f32>) -> Self {
        let context = Context::default();
        let state = State::new(
            context,
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            Some(Theme::Light),
            Some(2 * 1024),
        );
        let renderer = Renderer::new(
            device,
            TextureFormat::Rgba8UnormSrgb,
            RendererOptions::default(),
        );
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

    pub fn ppp(&mut self, v: f32) {
        self.state.egui_ctx().set_pixels_per_point(v);
    }

    pub fn begin_frame(&mut self, window: &Window, average_frame_time: f32) {
        let raw_input = self.state.take_egui_input(window);
        let context = self.state.egui_ctx();
        context.begin_pass(raw_input);

        Area::new("debug_widget".into())
            .anchor(Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .show(context, |ui| {
                Frame::new()
                    .fill(Color32::LIGHT_GRAY)
                    .inner_margin(Margin::same(10))
                    .corner_radius(CornerRadius {
                        nw: 10,
                        ..Default::default()
                    })
                    .shadow(Shadow {
                        color: Color32::ORANGE,
                        spread: 2,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        let style = ui.style_mut();
                        style.text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(16.0, egui::FontFamily::Monospace),
                        );

                        // ui.set_min_size([120.0, 0.0].into());
                        // ui.set_min_width(120.0);

                        egui::Grid::new("stats_grid").num_columns(2).show(ui, |ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label("FPS:");
                            });
                            let fps = (60.0 / (average_frame_time * 1000.0)).to_string();
                            ui.label(fps);
                            ui.end_row();

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label("X:");
                            });
                            ui.label("2");
                            ui.end_row();

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label("Y:");
                            });
                            ui.label("3");
                            ui.end_row();

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label("Z:");
                            });
                            ui.label("4");
                            ui.end_row();
                        });
                    });
            });

        self.frame_started = true;
    }

    pub fn end_frame_and_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        window: &Window,
        window_surface_view: &TextureView,
    ) {
        if !self.frame_started {
            panic!("Begin_frame must be called before end_frame_and_draw can be called!");
        }

        self.ppp(self.screen_descriptor.pixels_per_point);

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
            .update_buffers(device, queue, encoder, &tris, &self.screen_descriptor);
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
            .render(&mut rpass.forget_lifetime(), &tris, &self.screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        self.frame_started = false;
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.screen_descriptor = ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: 1.0,
        }
    }
}
