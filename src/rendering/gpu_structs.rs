use wgpu::BufferDescriptor;

use crate::model::camera::Camera;

pub trait Bufferable<'a> {
    fn desc() -> wgpu::BufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Parameters {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub dispatch_cnt: i32,

    pub reset_points_state: i32,
    pub invocation_iters: i32,
    pub padding_1: u32,
    pub padding_2: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct CameraStruct
{
    pub view_proj_mat: [[f32; 4]; 4], //mat4x4<f32>
    pub position: [f32; 4], //vec4<f32>
    pub forward: [f32; 4],
    pub focus_point: [f32; 4],

    pub aperture: f32,
    pub focus_distance: f32,
    pub depth_of_field: f32,
    pub projection_type: i32,
}

impl Default for CameraStruct {
    fn default() -> Self {
        Camera::default().create_camera_struct()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Iterator {
    pub color_speed: f32,
    pub color_index: f32,
    pub opacity: f32,
    pub reset_prob: f32,

    pub reset_alias: i32,
    pub tf_id: i32,
    pub real_params_index: i32,
    pub vec3_params_index: i32,

    pub shading_mode: i32,//0: default, 1: delta_p
    pub tf_mix: f32,
    pub tf_add: f32,
    pub padding2: i32,
}

impl Iterator {
    pub fn new() -> Self {
        Self {
            color_speed: 1.0,
            color_index: 1.0,
            opacity: 1.0,
            reset_prob: 1.0,
            reset_alias: 1,
            tf_id: 1,
            real_params_index: 1,
            vec3_params_index: 1,
            shading_mode: 1,
            tf_mix: 1.0,
            tf_add: 1.0,
            padding2: 0,
        }
    }
}

impl <'a> Bufferable<'a> for Iterator {
    fn desc() -> BufferDescriptor<'a> {
        BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Iterator>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Settings {
    pub camera_params: CameraStruct,

    pub fog_effect: f32,
    pub itnum : u32, //number of iterators
    pub palettecnt : i32,
    pub mark_area_in_focus: i32,

    pub warmup: u32,
    pub entropy: f32,
    pub max_filter_radius: i32,
    pub padding0: i32,

    pub filter_method: i32,
    pub filter_param0: f32,
    pub filter_param1: f32,
    pub filter_param2: f32,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            camera_params: CameraStruct::default(),
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
        }
    }
}

impl <'a> Bufferable<'a> for Settings {
    fn desc() -> BufferDescriptor<'a> {
        BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Settings>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }
    }
}
