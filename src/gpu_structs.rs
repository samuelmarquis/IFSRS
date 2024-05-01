use wgpu::BufferDescriptor;

pub trait Bufferable<'a> {
    fn desc() -> wgpu::BufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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

unsafe impl bytemuck::Zeroable for CameraStruct {}
unsafe impl bytemuck::Pod for CameraStruct {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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

unsafe impl bytemuck::Zeroable for Iterator {}
unsafe impl bytemuck::Pod for Iterator {}

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
#[derive(Copy, Clone, Debug)]
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

unsafe impl bytemuck::Zeroable for Settings {}
unsafe impl bytemuck::Pod for Settings {}

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
