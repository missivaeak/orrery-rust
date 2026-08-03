use cgmath::num_traits::ToPrimitive;
use egui::{
    Align, Align2, Area, Color32, Context, CornerRadius, Frame, Layout, Margin, Pos2, Rect, Shadow,
    Slider, Ui, ViewportId,
};
use egui_extras::{Size, Strip, StripBuilder};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, TextureFormat, TextureView};
use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    window::{Theme, Window},
};

use crate::helpers::entity::UpdateDescriptor;

pub struct Gui {
    pub state: State,
    renderer: Renderer,
    frame_started: bool,
    screen_descriptor: ScreenDescriptor,
    average_frame_ms: Option<f32>,
    rects: Vec<Rect>,
    wireframe_enabled: bool,
    lod_probe_enabled: bool,
    lod_distance_threshold: f32,
}

#[allow(unused, dead_code)]
#[derive(Clone)]
pub struct GuiUpdateDescriptor {
    pub wireframe_enabled: bool,
    pub lod_probe_enabled: bool,
    pub lod_distance_threshold: f32,
}

impl Default for GuiUpdateDescriptor {
    fn default() -> Self {
        Self {
            wireframe_enabled: true,
            lod_probe_enabled: true,
            lod_distance_threshold: 1.0,
        }
    }
}

impl Gui {
    pub fn new(
        device: &Device,
        window: &Window,
        size: LogicalSize<f32>,
        texture_format: TextureFormat,
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
        let renderer = Renderer::new(device, texture_format, RendererOptions::default());
        Self {
            renderer,
            state,
            screen_descriptor: ScreenDescriptor {
                size_in_pixels: [size.width as u32, size.height as u32],
                pixels_per_point: 1.0,
            },
            frame_started: false,
            average_frame_ms: None,
            rects: Vec::with_capacity(10),
            wireframe_enabled: true,
            lod_probe_enabled: true,
            lod_distance_threshold: 1.0,
        }
    }

    pub fn ppp(&mut self, v: f32) {
        self.state.egui_ctx().set_pixels_per_point(v);
    }

    pub fn set_frame_ms(&mut self, frame_ms: f32) {
        self.average_frame_ms = Some(frame_ms);
    }

    pub fn is_intersecting(&self, position: &PhysicalPosition<f64>) -> bool {
        let pos = Pos2 {
            x: position.x.to_f32().unwrap(),
            y: position.y.to_f32().unwrap(),
        };
        for rect in self.rects.iter() {
            if rect.contains(pos) {
                return true;
            }
        }
        false
    }

    pub fn get_update_descriptor(&self) -> GuiUpdateDescriptor {
        GuiUpdateDescriptor {
            wireframe_enabled: self.wireframe_enabled,
            lod_probe_enabled: self.lod_probe_enabled,
            lod_distance_threshold: self.lod_distance_threshold,
        }
    }

    pub fn begin_frame(&mut self, window: &Window, update_descriptor: &UpdateDescriptor) {
        let raw_input = self.state.take_egui_input(window);
        let context = self.state.egui_ctx();
        context.begin_pass(raw_input);

        let debug_widget = Area::new("debug_widget".into())
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
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::new(16.0, egui::FontFamily::Monospace),
                        );
                        ui.set_min_size([120.0, 0.0].into());

                        let (x, y, z) = (
                            format!("{:.1}", update_descriptor.controls.camera_position.x),
                            format!("{:.1}", update_descriptor.controls.camera_position.y),
                            format!("{:.1}", update_descriptor.controls.camera_position.z),
                        );

                        let fps = if let Some(ms) = self.average_frame_ms {
                            format!("{:.1}", (ms / 1000.0).powi(-1))
                        } else {
                            "-".to_owned()
                        };

                        let tri_count = update_descriptor
                            .renderer
                            .as_ref()
                            .map(|r| r.tri_count)
                            .unwrap_or(0);

                        let row_height = 22.0;
                        let col_width = 60.0;

                        StripBuilder::new(ui)
                            .sizes(Size::exact(row_height), 8)
                            .vertical(|mut strip| {
                                // Header
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .sizes(Size::exact(col_width), 4)
                                        .horizontal(|mut strip| {
                                            strip.cell(|_| {});
                                            strip.cell(|ui| {
                                                ui.label("X/R");
                                            });
                                            strip.cell(|ui| {
                                                ui.label("Y/F");
                                            });
                                            strip.cell(|ui| {
                                                ui.label("Z/U");
                                            });
                                        });
                                });

                                // Position
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .sizes(Size::exact(col_width), 4)
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label("Pos:");
                                                    },
                                                );
                                            });
                                            strip.cell(|ui| {
                                                ui.label(&x);
                                            });
                                            strip.cell(|ui| {
                                                ui.label(&y);
                                            });
                                            strip.cell(|ui| {
                                                ui.label(&z);
                                            });
                                        });
                                });

                                // FPS
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::exact(col_width))
                                        .size(Size::remainder())
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label("FPS:");
                                                    },
                                                );
                                            });
                                            strip.cell(|ui| {
                                                ui.label(&fps);
                                            });
                                        });
                                });

                                // Triangles
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::exact(col_width))
                                        .size(Size::remainder())
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label("Tri:");
                                                    },
                                                );
                                            });
                                            strip.cell(|ui| {
                                                ui.label(tri_count.to_string());
                                            });
                                        });
                                });

                                // Speed
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::exact(col_width))
                                        .size(Size::remainder())
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label("Speed:");
                                                    },
                                                );
                                            });
                                            strip.cell(|ui| {
                                                ui.label(format!(
                                                    "{:.1}",
                                                    update_descriptor.controls.speed * 100.0
                                                ));
                                            });
                                        });
                                });

                                // LOD (label + spanning slider)
                                strip.cell(|ui| {
                                    StripBuilder::new(ui)
                                        .size(Size::exact(col_width))
                                        .size(Size::remainder())
                                        .horizontal(|mut strip| {
                                            strip.cell(|ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.label("LOD:");
                                                    },
                                                );
                                            });
                                            strip.cell(|ui| {
                                                ui.add_sized(
                                                    ui.available_size(),
                                                    Slider::new(
                                                        &mut self.lod_distance_threshold,
                                                        0.5..=1.5,
                                                    ),
                                                );
                                            });
                                        });
                                });

                                // Full-width rows
                                strip.cell(|ui| {
                                    ui.checkbox(&mut self.wireframe_enabled, "Mesh wireframe");
                                });

                                strip.cell(|ui| {
                                    ui.checkbox(&mut self.lod_probe_enabled, "LOD Probe");
                                });
                            });
                    });
            });

        self.rects = [debug_widget.response.rect].to_vec();
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
