use std::mem::size_of;
use std::num::NonZeroU32;
use std::sync::mpsc::SyncSender;
use egui::TextureId;
use egui_wgpu::RenderState;
use wgpu::*;
use wgpu::BufferBindingType::{Storage, Uniform};
use wgpu::TextureFormat::Rgba8UnormSrgb;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::rendering::gpu_structs::*;
use crate::rendering::pipeline_compute::*;
use crate::rendering::pipeline_render::Render;


pub struct GraphicsEngine {
    pub compute_pipeline: Compute,
    pub render_pipeline: Render,
    pub shader: ShaderModule,

    tx: SyncSender<()>,

    pub(crate) output_texture: TextureId
}

// create texture view, create shader, create render_pipeline,
// per frame, create encoder, create render pass (can this be reused?), and submit to queue

pub const HISTOGRAM_WIDTH: usize = 1920;
pub const HISTOGRAM_HEIGHT: usize = 1080;
pub const WORKGROUP_SIZE: usize = 256;
pub const MAX_ITERATORS : usize =	100;
pub const MAX_PARAMS : usize = (2 * MAX_ITERATORS);
pub const MAX_PALETTE_COLORS : usize = 256;
pub const MAX_XAOS : usize = (MAX_ITERATORS * MAX_ITERATORS);

impl GraphicsEngine {
    pub fn new_engine(wgpu: &RenderState, tx: SyncSender<()>) -> Self {
        let shader_desc = wgpu::include_wgsl!("ifs_kernel.wgsl");
        let shader = wgpu.device.create_shader_module(shader_desc);

        let compute = Compute::init(wgpu, &shader);

        // TODO: unfuck that lol
        let render = Render::init(wgpu, &shader, compute.bind_group_layout.clone(), compute.bind_group.clone(), (1920, 1080));

        let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &render.texture_view, FilterMode::Nearest);

        Self {
            compute_pipeline: compute,
            render_pipeline: render,
            output_texture: tex_id,
            shader,
            tx,
        }
    }

    pub fn render(&self, wgpu: &RenderState) {
        let compute_cmd = self.compute_pipeline.encode_commands(wgpu);
        let render_cmd = self.render_pipeline.encode_commands(wgpu);

        wgpu.queue.submit([compute_cmd, render_cmd]);

        let moved_tx = self.tx.clone();
        wgpu.queue.on_submitted_work_done(move || moved_tx.send(()).unwrap() )
    }
}
