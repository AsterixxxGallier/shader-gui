use crate::gui::viewport::Viewport;
use eframe::egui_wgpu::{self, wgpu};
use eframe::wgpu::wgt::CommandEncoderDescriptor;
use eframe::wgpu::{BufferAddress, BufferUsages, ComputePassDescriptor, PrimitiveTopology};
use egui::{vec2, Response, Ui, Vec2, Widget};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

// 2 pixels per point
const WIDTH: usize = 1200 * 2;
const HEIGHT: usize = 800 * 2;

// TODO: Entirely dissolve this struct.
pub struct Renderer {
    offset: Vec2,
    zoom: f32,
}

fn load_shader_source(path: impl AsRef<Path>) -> wgpu::ShaderSource<'static> {
    let file = File::open(path).expect("could not open shader source file");
    let mut reader = BufReader::new(file);
    let mut source = String::new();
    reader.read_to_string(&mut source).expect("failed to read contents of shader source file");
    wgpu::ShaderSource::Wgsl(source.into())
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    pub fn load(render_state: &egui_wgpu::RenderState) {
        let device = &render_state.device;

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute"),
            source: load_shader_source("compute.wgsl"),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&compute_bind_group_layout)],
                immediate_size: 0,
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let compute_uniform_buffer_size = 6 * 4;

        let compute_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: compute_uniform_buffer_size as BufferAddress,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let storage_buffer_size = WIDTH * HEIGHT * 8;

        let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: storage_buffer_size as BufferAddress,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage_buffer.as_entire_binding(),
                },
            ],
        });

        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vertex"),
            source: load_shader_source("vertex.wgsl"),
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fragment"),
            source: load_shader_source("fragment.wgsl"),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: None,
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: None,
                targets: &[Some(render_state.target_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let uniform_buffer_size = 4 * 4;

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_buffer_size as BufferAddress,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: storage_buffer.as_entire_binding(),
                },
            ],
        });

        // Because the graphics pipeline must have the same lifetime as the egui render pass,
        // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
        // `paint_callback_resources` type map, which is stored alongside the render pass.
        render_state
            .renderer
            .write()
            .callback_resources
            .insert(RenderResources {
                pipeline,
                compute_pipeline,
                bind_group,
                compute_bind_group,
                uniform_buffer,
                compute_uniform_buffer,
            });
    }
}

impl Widget for &mut Renderer {
    fn ui(self, ui: &mut Ui) -> Response {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("The triangle is being painted using ");
                        ui.hyperlink_to("WGPU", "https://wgpu.rs");
                        ui.label(" (Portable Rust graphics API awesomeness)");
                    });

                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        Viewport::new(vec2(1200.0, 800.0)).show(
                            ui,
                            &mut self.offset,
                            &mut self.zoom,
                            |ui, rect, offset, zoom| {
                                // egui measures things in logical pixels, but the GPU uses physical
                                // pixels; this is the conversion factor
                                let pixels_per_point = ui.painter().pixels_per_point();

                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    CustomTriangleCallback {
                                        viewport_offset_x: rect.left() * pixels_per_point,
                                        viewport_offset_y: rect.top() * pixels_per_point,
                                        offset_x: (offset.x + rect.left()) * pixels_per_point,
                                        offset_y: (offset.y + rect.top()) * pixels_per_point,
                                        pixel_size: 1.0
                                            / (rect.height() * pixels_per_point * zoom),
                                        aspect_ratio: rect.aspect_ratio(),
                                    },
                                ));

                                ui.label(format!("zoom: {:.2}", zoom));
                            },
                        )
                    });
                    ui.label("Drag to rotate!");
                });
            })
            .response
    }
}

// Callbacks in egui_wgpu have 3 stages:
// * prepare (per callback impl)
// * finish_prepare (once)
// * paint (per callback impl)
//
// The prepare callback is called every frame before paint and is given access to the wgpu
// Device and Queue, which can be used, for instance, to update buffers and uniforms before
// rendering.
// If [`egui_wgpu::Renderer`] has [`egui_wgpu::FinishPrepareCallback`] registered,
// it will be called after all `prepare` callbacks have been called.
// You can use this to update any shared resources that need to be updated once per frame
// after all callbacks have been processed.
//
// On both prepare methods you can use the main `CommandEncoder` that is passed-in,
// return an arbitrary number of user-defined `CommandBuffer`s, or both.
// The main command buffer, as well as all user-defined ones, will be submitted together
// to the GPU in a single call.
//
// The paint callback is called after finish prepare and is given access to egui's main render pass,
// which can be used to issue draw commands.
struct CustomTriangleCallback {
    viewport_offset_x: f32,
    viewport_offset_y: f32,
    offset_x: f32,
    offset_y: f32,
    pixel_size: f32,
    aspect_ratio: f32,
}

impl egui_wgpu::CallbackTrait for CustomTriangleCallback {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _screen_descriptor: &egui_wgpu::ScreenDescriptor,
        _egui_encoder: &mut wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let resources: &RenderResources = resources.get().unwrap();
        resources.prepare(
            device,
            queue,
            self.viewport_offset_x,
            self.viewport_offset_y,
            self.offset_x,
            self.offset_y,
            self.pixel_size,
            self.aspect_ratio,
        );
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'static>,
        resources: &egui_wgpu::CallbackResources,
    ) {
        let resources: &RenderResources = resources.get().unwrap();
        resources.paint(render_pass);
    }
}

struct RenderResources {
    pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    compute_bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    compute_uniform_buffer: wgpu::Buffer,
}

impl RenderResources {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_offset_x: f32,
        viewport_offset_y: f32,
        offset_x: f32,
        offset_y: f32,
        pixel_size: f32,
        aspect_ratio: f32,
    ) {
        // Update our uniform buffer with the angle from the UI
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[
                viewport_offset_x.to_bits(),
                viewport_offset_y.to_bits(),
                WIDTH as u32,
                HEIGHT as u32,
            ]),
        );
        queue.write_buffer(
            &self.compute_uniform_buffer,
            0,
            bytemuck::cast_slice(&[
                offset_x.to_bits(),
                offset_y.to_bits(),
                pixel_size.to_bits(),
                aspect_ratio.to_bits(),
                WIDTH as u32,
                HEIGHT as u32,
            ]),
        );

        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&self.compute_pipeline);
        compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
        compute_pass.dispatch_workgroups(
            (WIDTH as u32).div_ceil(16),
            (HEIGHT as u32).div_ceil(16),
            1,
        );
        drop(compute_pass);
        queue.submit([command_encoder.finish()]);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
