use crate::helpers::constants::BACKGROUND_COLOUR;
use crate::helpers::rendering::{Object, Vertex};
use crate::scene::{GlobalFragmentUniform, GlobalVertexUniform};
use std::{borrow::Cow, sync::Arc};

use egui_wgpu::wgpu::Instance;
use std::collections::HashMap;
use wgpu::{
    Backends, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendComponent, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites, CompareFunction,
    CurrentSurfaceTexture, DepthBiasState, DepthStencilState, Device, DeviceDescriptor, Extent3d,
    Features, FragmentState, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PowerPreference, PresentMode,
    PrimitiveState, PrimitiveTopology, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StencilState, StoreOp,
    SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    VertexState,
    wgt::{CommandEncoderDescriptor, TextureViewDescriptor},
};
use wgpu::{CommandEncoder, PolygonMode, Queue, TextureView};
use winit::dpi::PhysicalSize;
use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum RenderGroupType {
    // Clear,
    Unlit,
    Lit,
}

impl RenderGroupType {
    const VALUES: [Self; 2] = [Self::Lit, Self::Unlit];
}

struct RenderGroup {
    render_pipeline: RenderPipeline,
    uniform_bind_group_layout: BindGroupLayout,
    global_vertex_uniform_buffer: Buffer,
    global_fragment_uniform_buffer: Buffer,
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_group_map: HashMap<RenderGroupType, RenderGroup>,
    wireframe_render_group: RenderGroup,
}

impl Renderer {
    pub async fn new(window: Arc<Window>, display_handle: Box<OwnedDisplayHandle>) -> Self {
        print_adapter_info(display_handle.clone()).await;

        let size = window.inner_size();
        // let instance_description = InstanceDescriptor {
        //     backends: Backends::all(),
        //     ..Default::default()
        // };
        let instance_descriptor = InstanceDescriptor::new_with_display_handle(display_handle);
        let instance = Instance::new(instance_descriptor);
        let surface = instance
            .create_surface(window)
            .expect("Failed to create surface");
        let request_adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };

        let adapter = instance
            .request_adapter(&request_adapter_options)
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::POLYGON_MODE_LINE,
                required_limits: Limits::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let format = surface_capabilities.formats[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let mut render_group_map: HashMap<RenderGroupType, RenderGroup> = HashMap::new();
        render_group_map.insert(
            RenderGroupType::Unlit,
            get_unlit_render_group(&device, &config),
        );
        render_group_map.insert(RenderGroupType::Lit, get_lit_render_group(&device, &config));
        // render_group_map.insert(
        //     RenderGroupType::Clear,
        //     get_clear_render_group(&device, &config),
        // );
        let wireframe_render_group = get_wireframe_render_group(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_group_map,
            wireframe_render_group,
        }
    }

    pub fn get_texture_format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn render<F>(
        &mut self,
        objects: &Vec<Object>,
        global_vertex_uniform: &GlobalVertexUniform,
        global_fragment_uniform: &GlobalFragmentUniform,
        mut egui_render: F,
        render_wireframes: bool,
    ) where
        F: FnMut(&Device, &Queue, &mut CommandEncoder, &TextureView),
    {
        let mut object_map: HashMap<RenderGroupType, Vec<&Object>> = HashMap::new();

        for object in objects {
            object_map
                .entry(object.render_group_type)
                .or_default()
                .push(object);
        }
        // access

        if let CurrentSurfaceTexture::Success(frame) = self.surface.get_current_texture() {
            let view = frame.texture.create_view(&TextureViewDescriptor::default());

            let depth_texture = self.device.create_texture(&TextureDescriptor {
                view_formats: &[],
                size: Extent3d {
                    width: self.size.width,
                    height: self.size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth24Plus,
                usage: TextureUsages::RENDER_ATTACHMENT,
                label: None,
            });
            let depth_view = depth_texture.create_view(&TextureViewDescriptor::default());
            let mut first = true;
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None });
            for render_group_type in RenderGroupType::VALUES {
                let render_group = self
                    .render_group_map
                    .get(&render_group_type)
                    .expect("Failed to get render group");

                let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: if first {
                                LoadOp::Clear(Color {
                                    r: BACKGROUND_COLOUR.x as f64,
                                    g: BACKGROUND_COLOUR.y as f64,
                                    b: BACKGROUND_COLOUR.z as f64,
                                    a: BACKGROUND_COLOUR.w as f64,
                                })
                            } else {
                                LoadOp::Load
                            },
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(Operations {
                            load: if first {
                                LoadOp::Clear(1.0)
                            } else {
                                LoadOp::Load
                            },
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                first = false;

                self.queue.write_buffer(
                    &render_group.global_vertex_uniform_buffer,
                    0,
                    bytemuck::bytes_of(global_vertex_uniform),
                );
                self.queue.write_buffer(
                    &render_group.global_fragment_uniform_buffer,
                    0,
                    bytemuck::bytes_of(global_fragment_uniform),
                );

                if let Some(objects) = object_map.get(&render_group_type) {
                    for object in objects.iter() {
                        {
                            let uniform_bind_group =
                                self.device.create_bind_group(&BindGroupDescriptor {
                                    layout: &render_group.uniform_bind_group_layout,
                                    entries: &[
                                        BindGroupEntry {
                                            binding: 0,
                                            resource: render_group
                                                .global_vertex_uniform_buffer
                                                .as_entire_binding(),
                                        },
                                        BindGroupEntry {
                                            binding: 1,
                                            resource: render_group
                                                .global_fragment_uniform_buffer
                                                .as_entire_binding(),
                                        },
                                        BindGroupEntry {
                                            binding: 2,
                                            resource: object
                                                .vertex_uniform_buffer
                                                .as_entire_binding(),
                                        },
                                        BindGroupEntry {
                                            binding: 3,
                                            resource: object
                                                .fragment_uniform_buffer
                                                .as_entire_binding(),
                                        },
                                    ],
                                    label: Some("Uniform Bind Group"),
                                });

                            for mesh in object.meshes.iter() {
                                rpass.set_pipeline(&render_group.render_pipeline);
                                rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                rpass.set_index_buffer(
                                    mesh.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint16,
                                );
                                rpass.set_bind_group(0, &uniform_bind_group, &[]);
                                rpass.draw_indexed(0..mesh.index_length, 0, 0..1);
                            }
                        }
                    }
                }
            }

            if render_wireframes {
                let mut wireframe_rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    // depth_stencil_attachment: None,
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });

                self.queue.write_buffer(
                    &self.wireframe_render_group.global_vertex_uniform_buffer,
                    0,
                    bytemuck::bytes_of(global_vertex_uniform),
                );
                self.queue.write_buffer(
                    &self.wireframe_render_group.global_fragment_uniform_buffer,
                    0,
                    bytemuck::bytes_of(global_fragment_uniform),
                );

                for object in objects.iter() {
                    let uniform_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
                        layout: &self.wireframe_render_group.uniform_bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: self
                                    .wireframe_render_group
                                    .global_vertex_uniform_buffer
                                    .as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: self
                                    .wireframe_render_group
                                    .global_fragment_uniform_buffer
                                    .as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: object.vertex_uniform_buffer.as_entire_binding(),
                            },
                            BindGroupEntry {
                                binding: 3,
                                resource: object.fragment_uniform_buffer.as_entire_binding(),
                            },
                        ],
                        label: Some("Uniform Bind Group"),
                    });

                    for mesh in object.meshes.iter() {
                        wireframe_rpass.set_pipeline(&self.wireframe_render_group.render_pipeline);
                        wireframe_rpass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        wireframe_rpass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        wireframe_rpass.set_bind_group(0, &uniform_bind_group, &[]);
                        wireframe_rpass.draw_indexed(0..mesh.index_length, 0, 0..1);
                    }
                }
            }

            egui_render(&self.device, &self.queue, &mut encoder, &view);

            self.queue.submit(Some(encoder.finish()));

            frame.present();
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

async fn print_adapter_info(display_handle: Box<OwnedDisplayHandle>) {
    let instance_descriptor = InstanceDescriptor::new_with_display_handle(display_handle);
    let instance = Instance::new(instance_descriptor);
    let adapters = instance.enumerate_adapters(Backends::all()).await;

    for adapter in adapters {
        println!("{:?}", adapter.get_info());
    }
}

fn get_unlit_render_group(device: &Device, config: &SurfaceConfiguration) -> RenderGroup {
    let source = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/unlit.wgsl")));
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source,
    });

    let global_vertex_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Vertex Uniform Buffer"),
        size: size_of::<GlobalVertexUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let global_fragment_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Fragment Uniform Buffer"),
        size: size_of::<GlobalFragmentUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("Unlit Uniform Bind Group Layout"),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[Some(&uniform_bind_group_layout)],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState {
                    color: BlendComponent::REPLACE,
                    alpha: BlendComponent::REPLACE,
                }),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            // cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::LessEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    RenderGroup {
        render_pipeline,
        uniform_bind_group_layout,
        global_vertex_uniform_buffer,
        global_fragment_uniform_buffer,
    }
}

fn get_lit_render_group(device: &Device, config: &SurfaceConfiguration) -> RenderGroup {
    let source = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/lit.wgsl")));
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source,
    });

    let global_vertex_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Vertex Uniform Buffer"),
        size: size_of::<GlobalVertexUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let global_fragment_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Fragment Uniform Buffer"),
        size: size_of::<GlobalFragmentUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("Unlit Uniform Bind Group Layout"),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[Some(&uniform_bind_group_layout)],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState {
                    color: BlendComponent::REPLACE,
                    alpha: BlendComponent::REPLACE,
                }),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            // cull_mode: Some(Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: Some(true),
            depth_compare: Some(CompareFunction::LessEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    RenderGroup {
        render_pipeline,
        uniform_bind_group_layout,
        global_vertex_uniform_buffer,
        global_fragment_uniform_buffer,
    }
}

// fn get_clear_render_group(device: &Device, config: &SurfaceConfiguration) -> RenderGroup {
//     let source = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/lit.wgsl")));
//     let shader = device.create_shader_module(ShaderModuleDescriptor {
//         label: None,
//         source,
//     });
//
//     let global_vertex_uniform_buffer = device.create_buffer(&BufferDescriptor {
//         label: Some("Global Vertex Uniform Buffer"),
//         size: size_of::<GlobalVertexUniform>() as u64,
//         usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
//         mapped_at_creation: false,
//     });
//
//     let global_fragment_uniform_buffer = device.create_buffer(&BufferDescriptor {
//         label: Some("Global Fragment Uniform Buffer"),
//         size: size_of::<GlobalFragmentUniform>() as u64,
//         usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
//         mapped_at_creation: false,
//     });
//
//     let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
//         entries: &[
//             BindGroupLayoutEntry {
//                 binding: 0,
//                 visibility: ShaderStages::VERTEX,
//                 ty: BindingType::Buffer {
//                     ty: BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             },
//             BindGroupLayoutEntry {
//                 binding: 1,
//                 visibility: ShaderStages::FRAGMENT,
//                 ty: BindingType::Buffer {
//                     ty: BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             },
//             BindGroupLayoutEntry {
//                 binding: 2,
//                 visibility: ShaderStages::VERTEX,
//                 ty: BindingType::Buffer {
//                     ty: BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             },
//             BindGroupLayoutEntry {
//                 binding: 3,
//                 visibility: ShaderStages::FRAGMENT,
//                 ty: BindingType::Buffer {
//                     ty: BufferBindingType::Uniform,
//                     has_dynamic_offset: false,
//                     min_binding_size: None,
//                 },
//                 count: None,
//             },
//         ],
//         label: Some("Unlit Uniform Bind Group Layout"),
//     });
//
//     let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
//         label: Some("Unlit Pipeline Layout"),
//         bind_group_layouts: &[Some(&uniform_bind_group_layout)],
//         immediate_size: 0,
//     });
//
//     let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
//         label: Some("Render Pipeline"),
//         layout: Some(&pipeline_layout),
//         vertex: VertexState {
//             module: &shader,
//             entry_point: Some("vs_main"),
//             buffers: &[Vertex::desc()],
//             compilation_options: PipelineCompilationOptions::default(),
//         },
//         fragment: Some(FragmentState {
//             module: &shader,
//             entry_point: Some("fs_main"),
//             targets: &[Some(ColorTargetState {
//                 format: config.format,
//                 blend: Some(BlendState {
//                     color: BlendComponent::REPLACE,
//                     alpha: BlendComponent::REPLACE,
//                 }),
//                 write_mask: ColorWrites::ALL,
//             })],
//             compilation_options: PipelineCompilationOptions::default(),
//         }),
//         primitive: PrimitiveState {
//             topology: PrimitiveTopology::TriangleList,
//             strip_index_format: None,
//             // cull_mode: Some(Face::Back),
//             ..Default::default()
//         },
//         depth_stencil: Some(DepthStencilState {
//             format: TextureFormat::Depth24Plus,
//             depth_write_enabled: Some(true),
//             depth_compare: Some(CompareFunction::LessEqual),
//             stencil: StencilState::default(),
//             bias: DepthBiasState::default(),
//         }),
//         multisample: MultisampleState::default(),
//         multiview_mask: None,
//         cache: None,
//     });
//
//     RenderGroup {
//         render_pipeline,
//         uniform_bind_group_layout,
//         global_vertex_uniform_buffer,
//         global_fragment_uniform_buffer,
//     }
// }

fn get_wireframe_render_group(device: &Device, config: &SurfaceConfiguration) -> RenderGroup {
    let source = ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/unlit.wgsl")));
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source,
    });

    let global_vertex_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Vertex Uniform Buffer"),
        size: size_of::<GlobalVertexUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let global_fragment_uniform_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("Global Fragment Uniform Buffer"),
        size: size_of::<GlobalFragmentUniform>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
        label: Some("Unlit Uniform Bind Group Layout"),
    });

    let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("Unlit Pipeline Layout"),
        bind_group_layouts: &[Some(&uniform_bind_group_layout)],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(ColorTargetState {
                format: config.format,
                blend: Some(BlendState {
                    color: BlendComponent::REPLACE,
                    alpha: BlendComponent::REPLACE,
                }),
                write_mask: ColorWrites::ALL,
            })],
            compilation_options: PipelineCompilationOptions::default(),
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Line,
            // strip_index_format: None,
            // cull_mode: Some(Face::Back),
            ..Default::default()
        },
        // depth_stencil: None,
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24Plus,
            depth_write_enabled: Some(false),
            depth_compare: Some(CompareFunction::LessEqual),
            stencil: StencilState::default(),
            bias: DepthBiasState {
                constant: -1,
                slope_scale: -1.0,
                clamp: 0.0,
            },
        }),
        multisample: MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    RenderGroup {
        render_pipeline,
        uniform_bind_group_layout,
        global_vertex_uniform_buffer,
        global_fragment_uniform_buffer,
    }
}
