use std::borrow::Cow;
use std::fs;
use std::iter::Iterator;
use std::mem::size_of;
use std::num::NonZeroU32;
use std::path::Iter;
use std::sync::mpsc::{Receiver, SyncSender, TryRecvError};
use std::thread::sleep;
use std::time::Duration;
use egui::TextureId;
use egui_wgpu::RenderState;
use futures::stream::iter;
use wgpu::*;
use wgpu::BufferBindingType::{Storage, Uniform};
use wgpu::TextureFormat::Rgba8UnormSrgb;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::model::ifs::IFS;
use crate::model::transform::Transform;
use crate::rendering::gpu_structs::*;
use crate::rendering::pipeline_compute::*;
use crate::rendering::pipeline_render::Render;


pub struct GraphicsEngine {
    pub compute_pipeline: Compute,
    pub render_pipeline: Render,
    pub shader: ShaderModule,

    work_status_tx: SyncSender<()>,
    ifs_rx: Receiver<IFS>,
    app_tx: SyncSender<TextureId>,
    dispatch_count: i32,
    model: IFS,
    // pub(crate) output_texture: TextureId
}

// create texture view, create shader, create render_pipeline,
// per frame, create encoder, create render pass (can this be reused?), and submit to queue

//TODO: NOT CONST
pub const HISTOGRAM_WIDTH: usize = 1920;
pub const HISTOGRAM_HEIGHT: usize = 1080;
pub const WORKGROUP_SIZE: usize = 256;
pub const MAX_ITERATORS : usize =	100;
pub const MAX_PARAMS : usize = (2 * MAX_ITERATORS);
pub const MAX_PALETTE_COLORS : usize = 256;
pub const MAX_XAOS : usize = (MAX_ITERATORS * MAX_ITERATORS);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Color([f32; 4]);

impl GraphicsEngine {
    pub fn new_engine(wgpu: &RenderState, work_status_tx: SyncSender<()>, ifs_rx: Receiver<IFS>, app_tx: SyncSender<TextureId>) -> Self {
        let shader_desc: ShaderModuleDescriptor<'_> = wgpu::include_wgsl!("ifs_kernel.wgsl");
        let shader = wgpu.device.create_shader_module(shader_desc);

        let compute = Compute::init(wgpu, &shader);

        // TODO: unfuck that lol
        let render = Render::init(wgpu, &shader, compute.bind_group_layout.clone(), compute.bind_group.clone(), (1920, 1080));

        let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &render.texture_view, FilterMode::Nearest);

        Self {
            compute_pipeline: compute,
            render_pipeline: render,
            shader,
            work_status_tx,
            ifs_rx,
            app_tx,
            dispatch_count: 0,
            model: Default::default(),
        }
    }

    pub fn render(&mut self, wgpu: &RenderState) {
        wgpu.queue.write_buffer(&self.compute_pipeline.done_buffer, 0 as BufferAddress, &vec![0u8; 4]);

        match self.ifs_rx.try_recv() {
            Ok(mut model) => {
                println!("updating model");
                self.update_model(wgpu, &mut model);
                self.model = model;
            }
            Err(e) => {}
        }

        // println!("dispatch: {}", self.dispatch_count);
        //

        wgpu.queue.write_buffer(&self.compute_pipeline.parameters_buffer, 0 as BufferAddress, bytemuck::cast_slice(&[ParametersStruct {
            seed: 699912576,
            width: self.model.width,
            height: self.model.height,
            dispatch_cnt: self.dispatch_count,
            reset_points_state: 0, // TODO: ??????
            invocation_iters: 512,
            padding_1: 0,
            padding_2: 0,
        }]));

        self.dispatch_count += 1;

        let compute_cmd = self.compute_pipeline.encode_commands(wgpu);
        let render_cmd = self.render_pipeline.encode_commands(wgpu);

        // let moved_tx = self.work_status_tx.clone();
        // moved_tx.send(()).unwrap();
        // wgpu.queue.on_submitted_work_done(move || moved_tx.send(()).unwrap());
        wgpu.queue.submit([compute_cmd, render_cmd]);

        // read one single bit
        loop {
            let buffer_slice = self.compute_pipeline.done_buffer.slice(..);
            let (sender, receiver) = futures::channel::oneshot::channel();
            buffer_slice.map_async(MapMode::Read, move |result| {
                sender.send(()).unwrap();
            });


            wgpu.device.poll(wgpu::Maintain::Wait);
            if let Ok(()) = futures::executor::block_on(receiver) {
                let data_view = buffer_slice.get_mapped_range();

                let x: &[u32] = bytemuck::cast_slice(&data_view);
                if x[0] == 1 {
                    break;
                }

                self.compute_pipeline.done_buffer.unmap();
            }
        }

        // this write will be submitted at the start of the next render?
    }

    fn update_model(&mut self, wgpu: &RenderState, model: &mut IFS) {
        // println!("{:?}", model.camera.create_camera_struct().view_proj_mat);

        if let Some(histogram_buffer) = self.reset_histogram(wgpu, model) {
            self.compute_pipeline.histogram_buffer = histogram_buffer;
            self.compute_pipeline.update_bind_group(wgpu);
            self.render_pipeline.bind_group = self.compute_pipeline.bind_group.clone();
        }

        // building the iterators creates a new shader
        self.build_iterators(wgpu, model);

        self.compute_pipeline.recreate_pipeline_with_shader(wgpu, &self.shader);
        self.render_pipeline.recreate_pipeline_with_shader(wgpu, &self.shader);

        // clear pstates
        wgpu.queue.write_buffer(&self.compute_pipeline.state_buffer, 0 as BufferAddress, &vec![0u8; size_of::<f32>() * 8 * crate::rendering::pipeline_compute::WORKGROUP_SIZE]);


        self.update_settings(wgpu, model);

        // update palette
        let color = Color([0.0, 0.0, 1.0, 0.001]);
        let colors = vec![color; MAX_PALETTE_COLORS];
        wgpu.queue.write_buffer(&self.compute_pipeline.palette_buffer, 0 as BufferAddress, &bytemuck::cast_slice(&colors));

        // update parameters
        wgpu.queue.write_buffer(&self.compute_pipeline.parameters_buffer, 0 as BufferAddress, bytemuck::cast_slice(&[ParametersStruct {
            seed: 699912576,
            width: model.width,
            height: model.height,
            dispatch_cnt: 0,
            reset_points_state: 0,
            invocation_iters: 0,
            padding_1: 0,
            padding_2: 0,
        }]));

        // resize
        self.render_pipeline.resize(wgpu, (model.width, model.height));
        let tex_id = wgpu.renderer.write().register_native_texture(&*wgpu.device, &self.render_pipeline.texture_view, FilterMode::Nearest);
        let _ = self.app_tx.try_send(tex_id).unwrap();
    }

    pub fn reset_histogram(&self, wgpu: &RenderState, model: &IFS) -> Option<Buffer>{
        // TODO: if has resize
        let hist_size = (model.width * model.height) as usize * size_of::<[f32;4]>();
        let newhist = vec![0; hist_size];
        if true {
            let new_hist = wgpu.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Histogram buffer"),
                contents: &newhist, // assuming RGBA8?
                usage: BufferUsages::STORAGE,
            });

            return Some(new_hist);
        } else {
            wgpu.queue.write_buffer(&self.compute_pipeline.histogram_buffer, 0 as BufferAddress, &newhist);
            return None;
        }
    }


    pub fn create_iterator_struct(i: &crate::model::iterator::Iterator, id: i32) -> IteratorStruct {
        IteratorStruct {
            color_speed: 0.0,
            color_index: 0.0,
            opacity: 0.0,
            reset_prob: 0.0,
            reset_alias: 0,
            tf_id: id,
            real_params_index: 0,
            vec3_params_index: 0,
            shading_mode: 0,
            tf_mix: 0.0,
            tf_add: 0.0,
            padding2: 0,
        }
    }
    //TODO NIGHTMARE NIGHTMARE NIGHTMARE NIGHTMARE
    fn build_iterators(&mut self, wgpu: &RenderState, model: &IFS) {
        let mut src_string = String::new();
        let mut iterators: Vec<IteratorStruct> = vec![];

        for (n, it) in model.iterators.iter().enumerate() {
            let mut real_vars = String::new();
            for (name, val) in &it.real_params {
                real_vars.push_str(&format!("\nvar {name} = {val};"))
            }

            let mut vec3_vars = String::new();
            for (name, val) in &it.vec3_params {
                vec3_vars.push_str(&format!("\nvar {name} = vec3({}, {}, {});", val[0], val[1], val[2]))
            }

            src_string.push_str(&format!("
                if (iter.tf_id == {n}) {{
                    {real_vars}
                    {vec3_vars}
                    {}
                }}", it.transform.source_code)
            );

            // Todo: never, lol
            iterators.push(IteratorStruct {
                color_speed: it.color_speed,
                color_index: it.color_index,
                opacity: it.opacity,
                reset_prob: 0.0,
                reset_alias: 0,
                tf_id: n as i32,
                real_params_index: 0, //MAKE ME REAAAAAAAAAL
                vec3_params_index: 0, // VEC ME DADDY
                shading_mode: 0,
                tf_mix: it.mix,
                tf_add: it.add,
                padding2: 0,
            });
        }

        // TODO: DO NOT REBUILD THE SHADER THIS MUCH
        let src = fs::read_to_string("src/rendering/ifs_kernel.wgsl").unwrap();

        // let src = include_str!("ifs_kernel.wgsl").to_owned();
        let modified_src = src.replace("@transforms", &src_string);

        let shader_desc = ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Owned(modified_src)),
        };
        let shader = wgpu.device.create_shader_module(shader_desc);

        self.shader = shader;

        // write iterators buffer
        wgpu.queue.write_buffer(&self.compute_pipeline.iterators_buffer, 0 as BufferAddress, &bytemuck::cast_slice(&iterators));
    }
    fn update_settings(&self, wgpu: &RenderState, model: &mut IFS) {
        let settings = SettingsStruct {
            camera_params: model.camera.create_camera_struct(),
            fog_effect: 0.0,
            itnum: model.iterators.len() as u32,
            palettecnt: MAX_PALETTE_COLORS as i32,
            mark_area_in_focus: 1,
            warmup: model.fuse,
            entropy: model.entropy as f32,
            max_filter_radius: 0,
            padding0: 0,
            filter_method: 0,
            filter_param0: 0.0,
            filter_param1: 0.0,
            filter_param2: 0.0
        };

        wgpu.queue.write_buffer(
            &self.compute_pipeline.settings_buffer,
            0 as BufferAddress,
            bytemuck::bytes_of(&settings)
        );

    }
}
