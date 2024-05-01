use std::num::NonZeroU32;
use egui::TextureId;
use egui_wgpu::RenderState;
use wgpu::{BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BufferDescriptor, CommandEncoderDescriptor, ComputePassDescriptor, ComputePipelineDescriptor, DynamicOffset, Extent3d, FilterMode, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline, ShaderModule, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor};
use wgpu::BufferBindingType::{Storage, Uniform};
use wgpu::TextureFormat::Rgba8UnormSrgb;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::gpu_structs::*;

pub struct GraphicsEngine {
    pipeline: RenderPipeline,
    texture_view: TextureView,
    pub(crate) output_texture: TextureId
}

// create texture view, create shader, create render_pipeline,
// per frame, create encoder, create render pass (can this be reused?), and submit to queue

impl GraphicsEngine {
    pub fn new_engine(wgpu: &RenderState) -> Self {
        let (pipeline, texture_view, output_texture, shader) = Self::init_render_pipeline(wgpu);

        let bd_wong = Settings::desc();
        println!("{:?}", bd_wong);
        println!("{}", std::mem::size_of::<Settings>());

        let settings_buffer = wgpu.device.create_buffer_init(
            &BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(
                    &Settings {
                        camera_params: CameraStruct {
                            view_proj_mat: [
                                [0.1, 0.2, 0.3, 0.4],
                                [0.0, 0.0, 0.0, 0.5],
                                [0.0, 0.0, 0.0, 0.6],
                                [6.9, 0.0, 0.0, 0.7],
                            ],
                            position: [0.0, 0.0, 0.0, 0.0],
                            forward: [0.0, 1.0, 0.0, 0.0],
                            focus_point: [0.0, 1.0, 0.0, 0.0],
                            aperture: 0.0,
                            focus_distance: 0.0,
                            depth_of_field: 0.0,
                            projection_type: 0,
                        },
                        fog_effect: 0.0,
                        itnum: 0,
                        palettecnt: 0,
                        mark_area_in_focus: 0,
                        warmup: 0,
                        entropy: 0.0,
                        max_filter_radius: 0,
                        padding0: 0,
                        filter_method: 0,
                        filter_param0: 0.0,
                        filter_param1: 0.0,
                        filter_param2: 0.0,
                    }),
                usage: Settings::desc().usage
            }
        );

        let group_layout = wgpu.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = wgpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(settings_buffer.as_entire_buffer_binding()),
                }
            ],
        });

        let layout = wgpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &group_layout,
            ],
            push_constant_ranges: &[

            ],
        });

        let compute_pipeline = wgpu.device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&layout),
            module: &shader,
            entry_point: "main",
        });

        // temp
        let mut encoder = wgpu.device.create_command_encoder(&CommandEncoderDescriptor {
            label: None,
        });

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&compute_pipeline);
            pass.set_bind_group(0, &bind_group, &[]); // TODO: find out what offsets means
            pass.dispatch_workgroups(4, 4, 4);
        }


        wgpu.queue.submit([encoder.finish()]);


        Self {
            pipeline,
            texture_view,
            output_texture
        }
    }

    // fn init_compute_pipeline()

    fn init_render_pipeline(wgpu: &RenderState) -> (RenderPipeline, TextureView, TextureId, ShaderModule) {
        let draw_tex = wgpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2, // 2d image
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST, // idfk, idc
            view_formats: &[],
        });

        let tex_view = draw_tex.create_view(
            &TextureViewDescriptor {
                format: Some(Rgba8UnormSrgb),
                ..Default::default()
            }
        );

        let shader_desc = wgpu::include_wgsl!("ifs_kernel.wgsl");
        let shader = wgpu.device.create_shader_module(shader_desc);

        let render_pipeline_layout =
            wgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = wgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[], // 2.
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &tex_view, FilterMode::Nearest);

        return (render_pipeline, tex_view, tex_id, shader)
    }

    pub fn render(&mut self, wgpu: &RenderState) {
        self.render_to_screen(wgpu);
    }

    pub fn do_compute(&mut self, wgpu: &RenderState) {

    }

    pub fn render_to_screen(&mut self, wgpu: &RenderState) {
        let mut encoder = wgpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("DEVIN RENDER Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }
                            ),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });


            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        };

        wgpu.queue.submit([encoder.finish()]);
    }
}