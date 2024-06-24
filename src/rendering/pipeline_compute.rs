use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;
use egui_wgpu::RenderState;
use wgpu::{BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
           BufferAddress, BufferDescriptor, BufferUsages, ComputePipelineDescriptor,
           PipelineLayoutDescriptor, ShaderStages};
use wgpu::BufferBindingType::{Storage, Uniform};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::rendering::gpu_structs::{IteratorStruct, Bufferable, SettingsStruct};
use wgpu::*;
use crate::rendering::graphics_engine::*;

pub const WORKGROUP_SIZE: usize = 256;

pub struct Compute {
    pub histogram_buffer: Buffer,
    pub state_buffer: Buffer,
    pub settings_buffer: Buffer,
    pub iterators_buffer: Buffer,
    pub alias_tables_buffer: Buffer,
    pub palette_buffer: Buffer,
    pub real_params_buffer: Buffer,
    pub vec3_params_buffer: Buffer,
    pub parameters_buffer: Buffer,
    pub next_sample_buffer: Buffer,

    pub bind_group_layout: Arc<BindGroupLayout>,
    pub bind_group: Arc<BindGroup>,

    // pub pipeline_layout: PipelineLayout,
    pub compute_pipeline: ComputePipeline,
    pub done_buffer: Buffer,
}

impl Compute {
    pub fn init(wgpu: &RenderState, shader: &ShaderModule) -> Self {
        let histogram_buffer = wgpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Histogram buffer"),
            contents: &vec![0u8; HISTOGRAM_WIDTH * HISTOGRAM_HEIGHT * size_of::<[f32;4]>()], // assuming RGBA8?
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let state_buffer = wgpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &vec![0u8; size_of::<f32>() * 8 * WORKGROUP_SIZE],
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST
        });

        let settings_buffer = wgpu.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&SettingsStruct::new()),
            usage: SettingsStruct::desc().usage | BufferUsages::COPY_DST
        });

        let iterators_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: wgpu::BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: MAX_ITERATORS as BufferAddress * crate::rendering::gpu_structs::IteratorStruct::desc().size,
            mapped_at_creation: false,
        });

        let alias_tables_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: (MAX_ITERATORS * size_of::<[f32; 4]>()) as BufferAddress,
            mapped_at_creation: false,
        });

        let palette_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: (MAX_PALETTE_COLORS * size_of::<[f32; 4]>()) as BufferAddress,
            mapped_at_creation: false,
        });

        let real_params_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: (16 * MAX_PARAMS) as BufferAddress,
            mapped_at_creation: false,
        });

        let vec3_params_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: (16 * MAX_PARAMS) as BufferAddress,
            mapped_at_creation: false,
        });

        let parameters_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            size: 32 as BufferAddress,
            mapped_at_creation: false,
        });

        let next_sample_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            size: size_of::<u32>() as BufferAddress,
            mapped_at_creation: false,
        });

        let done_buffer = wgpu.device.create_buffer(&BufferDescriptor {
            label: None,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            size: size_of::<u32>() as BufferAddress,
            mapped_at_creation: false,
        });

        let compute_pipeline = Self::create_pipeline_with(wgpu, &shader);
        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);

        println!("{:?}", bind_group_layout);

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(histogram_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(state_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(settings_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(iterators_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Buffer(alias_tables_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Buffer(palette_buffer.as_entire_buffer_binding())
                },
                // BindGroupEntry {
                //     binding: 6,
                //     resource: BindingResource::Buffer(real_params_buffer.as_entire_buffer_binding()),
                // },
                // BindGroupEntry {
                //     binding: 7,
                //     resource: BindingResource::Buffer(vec3_params_buffer.as_entire_buffer_binding())
                // },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Buffer(parameters_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::Buffer(next_sample_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::Buffer(done_buffer.as_entire_buffer_binding())
                }
            ],
        });

        Self {
            histogram_buffer,
            state_buffer,
            settings_buffer,
            iterators_buffer,
            alias_tables_buffer,
            palette_buffer,
            real_params_buffer,
            vec3_params_buffer,
            parameters_buffer,
            next_sample_buffer,
            done_buffer,

            bind_group: Arc::new(bind_group),
            bind_group_layout: Arc::new(bind_group_layout),

            compute_pipeline,
            // pipeline_layout: layout,
        }
    }

    pub fn update_bind_group(&mut self, wgpu: &RenderState) {
        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(self.histogram_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Buffer(self.state_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(self.settings_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Buffer(self.iterators_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 4,
                    resource: BindingResource::Buffer(self.alias_tables_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: BindingResource::Buffer(self.palette_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 6,
                    resource: BindingResource::Buffer(self.real_params_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 7,
                    resource: BindingResource::Buffer(self.vec3_params_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 8,
                    resource: BindingResource::Buffer(self.parameters_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 9,
                    resource: BindingResource::Buffer(self.next_sample_buffer.as_entire_buffer_binding())
                },
                BindGroupEntry {
                    binding: 10,
                    resource: BindingResource::Buffer(self.done_buffer.as_entire_buffer_binding())
                }
            ],
        });

        self.bind_group = Arc::new(bind_group);
    }

    pub fn create_pipeline_with(wgpu: &RenderState, shader: &ShaderModule) -> ComputePipeline {
        wgpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: "main",
            //compilation_options: Default::default(),
        })
    }

    pub fn recreate_pipeline_with_shader(&mut self, wgpu: &RenderState, shader: &ShaderModule) {
        self.compute_pipeline = Self::create_pipeline_with(wgpu, shader);
        self.bind_group_layout = Arc::new(self.compute_pipeline.get_bind_group_layout(0));
    }

    pub fn encode_commands(&self, wgpu: &RenderState) -> CommandBuffer {
        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: None, timestamp_writes: None });

            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(WORKGROUP_SIZE as u32, 1, 1);
        }

        encoder.finish()
    }
}
