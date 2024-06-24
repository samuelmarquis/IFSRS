use std::sync::Arc;
use egui_wgpu::RenderState;
use wgpu::*;
use wgpu::TextureFormat::Rgba8UnormSrgb;


pub struct Render {
    pub width: u32,
    pub height: u32,
    pub texture: Texture,
    pub texture_view: TextureView,
    pub pipeline_layout: PipelineLayout,
    pub pipeline: RenderPipeline,
    pub bind_group_layout: Arc<BindGroupLayout>,
    pub bind_group: Arc<BindGroup>,
}

impl Render {
    pub(crate) fn resize(&mut self, wgpu: &RenderState, size: (u32, u32)) {
        let (width, height) = size;

        let old_tex = std::mem::replace(&mut self.texture, wgpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2, // 2d image
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        self.texture_view = self.texture.create_view(
            &TextureViewDescriptor {
                format: Some(Rgba8UnormSrgb),
                ..Default::default()
            }
        );

        // TODO: figure out how to destroy texture >:(
        // ...safely.
        // EVIL CODE GNOME TODO: LEAK MORE MEMORY >:^3c
        //old_tex.destroy();
    }

    pub fn init(wgpu: &RenderState, shader: &ShaderModule, bind_group_layout: Arc<BindGroupLayout>, bind_group: Arc<BindGroup>, texture_size: (u32, u32)) -> Self {
        let (width, height) = texture_size;

        let draw_tex = wgpu.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2, // 2d image
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::COPY_DST, // idfk, idc
            view_formats: &[],
        });

        let render_pipeline_layout =
            wgpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = Self::create_pipeline_with(wgpu, &render_pipeline_layout, shader);

        let tex_view = draw_tex.create_view(
            &TextureViewDescriptor {
                format: Some(Rgba8UnormSrgb),
                ..Default::default()
            }
        );

        Self {
            width,
            height,
            texture: draw_tex,
            texture_view: tex_view,
            pipeline_layout: render_pipeline_layout,
            pipeline: render_pipeline,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn create_pipeline_with(wgpu: &RenderState, layout: &PipelineLayout, shader: &ShaderModule) -> RenderPipeline {
        wgpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                //compilation_options: Default::default(),
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
                //compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }


    pub fn recreate_pipeline_with_shader(&mut self, wgpu: &RenderState, shader: &ShaderModule) {
        self.pipeline = Self::create_pipeline_with(wgpu, &self.pipeline_layout, shader);
    }

    pub fn encode_commands(&self, wgpu: &RenderState) -> CommandBuffer {
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
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        };

        encoder.finish()
    }
}
