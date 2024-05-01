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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Settings {
    camera_params: CameraStruct,

    fog_effect: f32,
    itnum : u32, //number of iterators
    palettecnt : i32,
    mark_area_in_focus: i32,

    warmup: u32,
    entropy: f32,
    max_filter_radius: i32,
    padding0: i32,

    filter_method: i32,
    filter_param0: f32,
    filter_param1: f32,
    filter_param2: f32,
}

unsafe impl bytemuck::Zeroable for Settings {}
unsafe impl bytemuck::Pod for Settings {}