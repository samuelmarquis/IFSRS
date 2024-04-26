#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CameraStruct
{
    view_proj_mat: [[f32; 4]; 4], //saw someone on stackoverflow pack mat4x4 this way
    position: [f32; 4], //likewise
    forward: [f32; 4],
    focus_point: [f32; 4],

    aperture: f32,
    focus_distance: f32,
    depth_of_field: f32,
    projection_type: i32,
}

unsafe impl bytemuck::Zeroable for CameraStruct {}
unsafe impl bytemuck::Pod for CameraStruct {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Iterator {
    color_speed: f32,
    color_index: f32,
    opacity: f32,
    reset_prob: f32,

    reset_alias: i32,
    tfId: i32,
    real_params_index: i32,
    vec3_params_index: i32,

    shading_mode: i32,//0: default, 1: delta_p
    tf_mix: f32,
    tf_add: f32,
    padding2: i32,
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