use crate::gui::viewport::Viewport;
use eframe::egui_wgpu::{self, wgpu};
use eframe::wgpu::wgt::CommandEncoderDescriptor;
use eframe::wgpu::{BufferAddress, BufferUsages, ComputePassDescriptor, PrimitiveTopology};
use egui::{vec2, Response, Ui, Vec2, Widget};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use value::{Value, ValueType};
use crate::gui_gpu::id::{Id, IdMap};

const WIDTH: usize = 1200 * 2;
const HEIGHT: usize = 800 * 2;

pub mod id;
pub mod value;

pub struct PassUniformBufferDescriptor {
    pub binding: u32,
    pub name: String,
    pub value_type: ValueType,
}

pub struct PassStorageBufferDescriptor {
    pub binding: u32,
    pub name: String,
    pub element_type: ValueType,
    pub writable: bool,
}

pub struct PassDescriptor {
    pub id: Id,
    pub shader_source_path: PathBuf,
    pub name: String,
    pub uniform_buffer_descriptors: Vec<PassUniformBufferDescriptor>,
    pub storage_buffer_descriptors: Vec<PassStorageBufferDescriptor>,
}

pub struct PassUniformValue {
    pub binding: u32,
    pub value: Value,
}

pub struct PassStorageBufferBinding {
    pub binding: u32,
    pub buffer_id: Id,
}

pub struct PassInstanceDescriptor {
    pub pass_id: Id,
    pub storage_buffer_bindings: Vec<PassStorageBufferBinding>,
}

pub struct SharedBufferDescriptor {
    pub id: Id,
    pub name: String,
    pub element_type: ValueType,
}

pub struct PassSequenceDescriptor {
    pub passes: IdMap<PassDescriptor>,
    pub shared_buffers: IdMap<SharedBufferDescriptor>,
    pub pass_instances: Vec<PassInstanceDescriptor>,
    pub painted_buffer_id: Id,
}

pub struct Pass {
    /// corresponds to [`PassDescriptor::id`]
    pub id: Id,
    pub shader_module: wgpu::ShaderModule,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::ComputePipeline,
}

pub struct PassInstanceUniformBuffer {
    pub binding: u32,
    pub buffer: wgpu::Buffer,
}

pub struct SharedBuffer {
    /// corresponds to [`SharedBufferDescriptor::id`]
    pub id: Id,
    pub buffer: wgpu::Buffer,
}

pub struct PassInstance {
    pub pass_id: Id,
    pub uniform_values: Vec<PassUniformValue>,
    pub uniform_buffers: Vec<PassInstanceUniformBuffer>,
    pub bind_group: wgpu::BindGroup,
}

pub struct PassSequence {
    pub viewport_size_x: usize,
    pub viewport_size_y: usize,
    pub passes: IdMap<Pass>,
    pub shared_buffers: IdMap<SharedBuffer>,
    pub standard_uniform_buffer: wgpu::Buffer,
    pub pass_instances: Vec<PassInstance>,
    pub painted_buffer_id: Id,
}

fn create_bind_group_layout(
    device: &wgpu::Device,
    pass_descriptor: &PassDescriptor,
) -> wgpu::BindGroupLayout {
    // always available uniform buffer
    let mut bind_group_layout_entries = vec![wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }];

    for buffer_descriptor in &pass_descriptor.uniform_buffer_descriptors {
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: buffer_descriptor.binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    for buffer_descriptor in &pass_descriptor.storage_buffer_descriptors {
        bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: buffer_descriptor.binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage {
                    read_only: !buffer_descriptor.writable,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    }

    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &bind_group_layout_entries[..],
    })
}

fn create_shader_module(
    device: &wgpu::Device,
    pass_descriptor: &PassDescriptor,
) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&pass_descriptor.name),
        source: load_shader_source(&pass_descriptor.shader_source_path),
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    shader_module: &wgpu::ShaderModule,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader_module,
        entry_point: None,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn create_uniform_buffer(device: &wgpu::Device, size: BufferAddress) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    })
}

fn create_storage_buffer(device: &wgpu::Device, size: BufferAddress) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: BufferUsages::STORAGE,
        mapped_at_creation: false,
    })
}

fn create_pass_instance_uniform_buffers(
    device: &wgpu::Device,
    pass_descriptor: &PassDescriptor,
) -> Vec<PassInstanceUniformBuffer> {
    pass_descriptor
        .uniform_buffer_descriptors
        .iter()
        .map(|uniform_buffer_descriptor| {
            let buffer = create_uniform_buffer(
                device,
                uniform_buffer_descriptor.value_type.size() as BufferAddress,
            );
            PassInstanceUniformBuffer {
                binding: uniform_buffer_descriptor.binding,
                buffer,
            }
        })
        .collect()
}

fn create_shared_buffers(
    device: &wgpu::Device,
    shared_buffer_descriptors: &IdMap<SharedBufferDescriptor>,
    viewport_size_x: usize,
    viewport_size_y: usize,
) -> IdMap<SharedBuffer> {
    shared_buffer_descriptors
        .iter()
        .map(|(&id, shared_buffer_descriptor)| {
            let size =
                viewport_size_x * viewport_size_y * shared_buffer_descriptor.element_type.align();
            let buffer = create_storage_buffer(device, size as BufferAddress);
            (id, SharedBuffer { id, buffer })
        })
        .collect()
}

fn create_bind_group(
    device: &wgpu::Device,
    pass_descriptor: &PassDescriptor,
    pass_instance_descriptor: &PassInstanceDescriptor,
    bind_group_layout: &wgpu::BindGroupLayout,
    standard_uniform_buffer: &wgpu::Buffer,
    pass_instance_uniform_buffers: &[PassInstanceUniformBuffer],
    shared_buffers: &IdMap<SharedBuffer>,
) -> wgpu::BindGroup {
    let mut bind_group_entries = vec![wgpu::BindGroupEntry {
        binding: 0,
        resource: standard_uniform_buffer.as_entire_binding(),
    }];

    for (uniform_buffer_descriptor, buffer) in pass_descriptor
        .uniform_buffer_descriptors
        .iter()
        .zip(pass_instance_uniform_buffers)
    {
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: uniform_buffer_descriptor.binding,
            resource: buffer.buffer.as_entire_binding(),
        });
    }

    for storage_buffer_binding in &pass_instance_descriptor.storage_buffer_bindings {
        let buffer = &shared_buffers[&storage_buffer_binding.buffer_id].buffer;
        bind_group_entries.push(wgpu::BindGroupEntry {
            binding: storage_buffer_binding.binding,
            resource: buffer.as_entire_binding(),
        });
    }

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &bind_group_entries[..],
    })
}

fn create_passes(device: &wgpu::Device, pass_descriptors: &IdMap<PassDescriptor>) -> IdMap<Pass> {
    pass_descriptors
        .iter()
        .map(|(&id, pass_descriptor)| {
            let shader_module = create_shader_module(device, &pass_descriptor);
            let bind_group_layout = create_bind_group_layout(device, &pass_descriptor);
            let pipeline = create_pipeline(device, &shader_module, &bind_group_layout);
            (
                id,
                Pass {
                    id,
                    shader_module,
                    bind_group_layout,
                    pipeline,
                },
            )
        })
        .collect()
}

fn create_pass_uniform_values(pass_descriptor: &PassDescriptor) -> Vec<PassUniformValue> {
    pass_descriptor
        .uniform_buffer_descriptors
        .iter()
        .map(|uniform_buffer_descriptor| {
            let value = uniform_buffer_descriptor.value_type.default_value();
            PassUniformValue {
                binding: uniform_buffer_descriptor.binding,
                value,
            }
        })
        .collect()
}

fn create_pass_instances(
    device: &wgpu::Device,
    pass_instance_descriptors: &[PassInstanceDescriptor],
    standard_uniform_buffer: &wgpu::Buffer,
    shared_buffers: &IdMap<SharedBuffer>,
    pass_descriptors: &IdMap<PassDescriptor>,
    passes: &IdMap<Pass>,
) -> Vec<PassInstance> {
    pass_instance_descriptors
        .iter()
        .map(|pass_instance_descriptor| {
            let pass_id = pass_instance_descriptor.pass_id;
            let pass_descriptor = &pass_descriptors[&pass_id];
            let pass = &passes[&pass_id];
            let uniform_values = create_pass_uniform_values(pass_descriptor);
            let uniform_buffers = create_pass_instance_uniform_buffers(device, pass_descriptor);
            let bind_group = create_bind_group(
                device,
                pass_descriptor,
                pass_instance_descriptor,
                &pass.bind_group_layout,
                standard_uniform_buffer,
                &uniform_buffers,
                shared_buffers,
            );
            PassInstance {
                pass_id,
                uniform_values,
                uniform_buffers,
                bind_group,
            }
        })
        .collect()
}

fn create_sequence(
    device: &wgpu::Device,
    descriptor: &PassSequenceDescriptor,
    viewport_size_x: usize,
    viewport_size_y: usize,
) -> PassSequence {
    let standard_uniform_buffer = create_uniform_buffer(device, 8 * 4);
    let shared_buffers = create_shared_buffers(
        device,
        &descriptor.shared_buffers,
        viewport_size_x,
        viewport_size_y,
    );

    let passes = create_passes(device, &descriptor.passes);
    let pass_instances = create_pass_instances(
        device,
        &descriptor.pass_instances,
        &standard_uniform_buffer,
        &shared_buffers,
        &descriptor.passes,
        &passes,
    );

    PassSequence {
        viewport_size_x,
        viewport_size_y,
        passes,
        shared_buffers,
        standard_uniform_buffer,
        pass_instances,
        painted_buffer_id: descriptor.painted_buffer_id,
    }
}

fn create_paint_resources(
    device: &wgpu::Device,
    render_state: &egui_wgpu::RenderState,
    painted_buffer: &wgpu::Buffer,
) -> PaintResources {
    let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("vertex"),
        source: load_shader_source(VERTEX_SHADER_SOURCE_PATH),
    });

    let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fragment"),
        source: load_shader_source(FRAGMENT_SHADER_SOURCE_PATH),
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
                resource: painted_buffer.as_entire_binding(),
            },
        ],
    });

    PaintResources {
        pipeline,
        bind_group,
        uniform_buffer,
    }
}

pub fn load_render_resources(render_state: &egui_wgpu::RenderState) {
    let pixel_to_complex = PassDescriptor {
        id: Id::new(),
        shader_source_path: "shaders/pixel_to_complex.wgsl".into(),
        name: "Pixel coordinates to complex".to_string(),
        uniform_buffer_descriptors: vec![],
        storage_buffer_descriptors: vec![PassStorageBufferDescriptor {
            binding: 1,
            name: "Complex plane".to_string(),
            element_type: ValueType::Complex,
            writable: true,
        }],
    };
    let complex_function = PassDescriptor {
        id: Id::new(),
        shader_source_path: "shaders/complex_function.wgsl".into(),
        name: "Complex function".to_string(),
        uniform_buffer_descriptors: vec![],
        storage_buffer_descriptors: vec![
            PassStorageBufferDescriptor {
                binding: 1,
                name: "Input".to_string(),
                element_type: ValueType::Complex,
                writable: false,
            },
            PassStorageBufferDescriptor {
                binding: 2,
                name: "Output".to_string(),
                element_type: ValueType::Complex,
                writable: true,
            },
        ],
    };
    let complex_to_polar = PassDescriptor {
        id: Id::new(),
        shader_source_path: "shaders/complex_to_polar.wgsl".into(),
        name: "Complex to polar".to_string(),
        uniform_buffer_descriptors: vec![],
        storage_buffer_descriptors: vec![
            PassStorageBufferDescriptor {
                binding: 1,
                name: "Complex".to_string(),
                element_type: ValueType::Complex,
                writable: false,
            },
            PassStorageBufferDescriptor {
                binding: 2,
                name: "Polar".to_string(),
                element_type: ValueType::Polar,
                writable: true,
            },
        ],
    };
    let polar_to_color = PassDescriptor {
        id: Id::new(),
        shader_source_path: "shaders/polar_to_color.wgsl".into(),
        name: "Polar to color".to_string(),
        uniform_buffer_descriptors: vec![],
        storage_buffer_descriptors: vec![
            PassStorageBufferDescriptor {
                binding: 1,
                name: "Polar".to_string(),
                element_type: ValueType::Polar,
                writable: false,
            },
            PassStorageBufferDescriptor {
                binding: 2,
                name: "Color".to_string(),
                element_type: ValueType::RgbColor,
                writable: true,
            },
        ],
    };
    /*let flat_color = PassDescriptor {
        id: Id::new(),
        shader_source_path: "shaders/flat_color.wgsl".into(),
        name: "Flat color".to_string(),
        uniform_buffer_descriptors: vec![],
        storage_buffer_descriptors: vec![
            PassStorageBufferDescriptor {
                binding: 1,
                name: "Color".to_string(),
                element_type: ValueType::RgbColor,
                writable: true,
            },
        ],
    };*/
    let complex_buffer = SharedBufferDescriptor {
        id: Id::new(),
        name: "Complex buffer".to_string(),
        element_type: ValueType::Complex,
    };
    let complex_buffer_2 = SharedBufferDescriptor {
        id: Id::new(),
        name: "Complex buffer 2".to_string(),
        element_type: ValueType::Complex,
    };
    let polar_buffer = SharedBufferDescriptor {
        id: Id::new(),
        name: "Polar buffer".to_string(),
        element_type: ValueType::Polar,
    };
    let color_buffer = SharedBufferDescriptor {
        id: Id::new(),
        name: "Color buffer".to_string(),
        element_type: ValueType::RgbColor,
    };
    let pixel_to_complex_instance = PassInstanceDescriptor {
        pass_id: pixel_to_complex.id,
        storage_buffer_bindings: vec![PassStorageBufferBinding {
            binding: 1,
            buffer_id: complex_buffer.id,
        }],
    };
    let complex_function_instance = PassInstanceDescriptor {
        pass_id: complex_function.id,
        storage_buffer_bindings: vec![
            PassStorageBufferBinding {
                binding: 1,
                buffer_id: complex_buffer.id,
            },
            PassStorageBufferBinding {
                binding: 2,
                buffer_id: complex_buffer_2.id,
            },
        ],
    };
    let complex_to_polar_instance = PassInstanceDescriptor {
        pass_id: complex_to_polar.id,
        storage_buffer_bindings: vec![
            PassStorageBufferBinding {
                binding: 1,
                buffer_id: complex_buffer_2.id,
            },
            PassStorageBufferBinding {
                binding: 2,
                buffer_id: polar_buffer.id,
            },
        ],
    };
    let polar_to_color_instance = PassInstanceDescriptor {
        pass_id: polar_to_color.id,
        storage_buffer_bindings: vec![
            PassStorageBufferBinding {
                binding: 1,
                buffer_id: polar_buffer.id,
            },
            PassStorageBufferBinding {
                binding: 2,
                buffer_id: color_buffer.id,
            },
        ],
    };
    /*let flat_color_instance = PassInstanceDescriptor {
        pass_id: flat_color.id,
        storage_buffer_bindings: vec![
            PassStorageBufferBinding {
                binding: 1,
                buffer_id: color_buffer.id,
            },
        ],
    };*/
    let painted_buffer_id = color_buffer.id;
    let sequence_descriptor = PassSequenceDescriptor {
        passes: [pixel_to_complex, complex_to_polar, polar_to_color, complex_function]
            .into_iter()
            .map(|it| (it.id, it))
            .collect(),
        shared_buffers: vec![complex_buffer, complex_buffer_2, polar_buffer, color_buffer]
            .into_iter()
            .map(|it| (it.id, it))
            .collect(),
        pass_instances: vec![
            pixel_to_complex_instance,
            complex_function_instance,
            complex_to_polar_instance,
            polar_to_color_instance,
        ],
        painted_buffer_id,
    };

    let device = &render_state.device;

    let viewport_size_x = 2400;
    let viewport_size_y = 1600;

    let sequence = create_sequence(
        device,
        &sequence_descriptor,
        viewport_size_x,
        viewport_size_y,
    );
    let painted_buffer = &sequence.shared_buffers[&sequence.painted_buffer_id].buffer;
    let paint_resources = create_paint_resources(device, render_state, painted_buffer);

    render_state
        .renderer
        .write()
        .callback_resources
        .insert(RenderResources {
            paint_resources,
            sequence,
        });
}

pub const VERTEX_SHADER_SOURCE_PATH: &str = "shaders/vertex.wgsl";
pub const FRAGMENT_SHADER_SOURCE_PATH: &str = "shaders/fragment.wgsl";

fn load_shader_source(path: impl AsRef<Path>) -> wgpu::ShaderSource<'static> {
    let file = File::open(path.as_ref()).expect(&format!(
        "could not open shader source file at {:?}",
        path.as_ref()
    ));
    let mut reader = BufReader::new(file);
    let mut source = String::new();
    reader
        .read_to_string(&mut source)
        .expect("failed to read contents of shader source file");
    wgpu::ShaderSource::Wgsl(source.into())
}

// TODO: Entirely dissolve this struct.
pub struct Demo {
    offset: Vec2,
    zoom: f32,
}

impl Demo {
    pub fn new() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Widget for &mut Demo {
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
                            |ui, rect, offset, zoom, response| {
                                // egui measures things in logical pixels, but the GPU uses physical
                                // pixels; this is the conversion factor
                                let pixels_per_point = ui.painter().pixels_per_point();

                                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                                    rect,
                                    CustomTriangleCallback {
                                        viewport_offset_x: rect.left() * pixels_per_point,
                                        viewport_offset_y: rect.top() * pixels_per_point,
                                        offset_x: offset.x * pixels_per_point,
                                        offset_y: offset.y * pixels_per_point,
                                        pixel_size: 1.0 / (rect.height() * pixels_per_point * zoom),
                                        aspect_ratio: rect.aspect_ratio(),
                                        cursor_x: response.hover_pos().unwrap_or_default().x * pixels_per_point,
                                        cursor_y: response.hover_pos().unwrap_or_default().y * pixels_per_point,
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
    cursor_x: f32,
    cursor_y: f32,
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
            self.cursor_x,
            self.cursor_y,
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

struct PaintResources {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
}

struct RenderResources {
    paint_resources: PaintResources,
    sequence: PassSequence,
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
        cursor_x: f32,
        cursor_y: f32,
    ) {
        queue.write_buffer(
            &self.paint_resources.uniform_buffer,
            0,
            bytemuck::cast_slice(&[
                viewport_offset_x.to_bits(),
                viewport_offset_y.to_bits(),
                WIDTH as u32,
                HEIGHT as u32,
            ]),
        );
        queue.write_buffer(
            &self.sequence.standard_uniform_buffer,
            0,
            bytemuck::cast_slice(&[
                offset_x.to_bits(),
                offset_y.to_bits(),
                pixel_size.to_bits(),
                aspect_ratio.to_bits(),
                WIDTH as u32,
                HEIGHT as u32,
                cursor_x.to_bits(),
                cursor_y.to_bits(),
            ]),
        );

        let mut command_encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        for pass_instance in &self.sequence.pass_instances {
            let pass = &self.sequence.passes[&pass_instance.pass_id];
            let mut compute_pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&pass.pipeline);
            compute_pass.set_bind_group(0, &pass_instance.bind_group, &[]);
            compute_pass.dispatch_workgroups(
                (self.sequence.viewport_size_x as u32).div_ceil(16),
                (self.sequence.viewport_size_y as u32).div_ceil(16),
                1,
            );
        }
        queue.submit([command_encoder.finish()]);
    }

    fn paint(&self, render_pass: &mut wgpu::RenderPass<'_>) {
        // Draw our triangle!
        render_pass.set_pipeline(&self.paint_resources.pipeline);
        render_pass.set_bind_group(0, &self.paint_resources.bind_group, &[]);
        render_pass.draw(0..4, 0..1);
    }
}
