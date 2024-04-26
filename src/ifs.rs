use std::path::Path;

pub struct IFS{
    pub title: String,
    pub width : u16,
    pub height : u16,
    pub brightness: f32, //strictly >= 0
    pub gamma_inv: f32, //strictly >= 0
    pub gamma_thresh: f32, //strictly >= 0
    pub vibrancy: f32, //can be positive or negative
    pub background_color: [f32; 3],
    //3d settings
    pub fov: f32, //[1-180]
    pub aperture: f32, // strictly >= 0
    pub fdist: f32, //focus distance, can be positive or negative
    pub dof: f32, // strictly >= 0
    //render settings
    pub entropy: f32, // chance to reset on each iteration
    pub fuse: u16, // usually 20, number of iterations to discard before plotting
    pub pause_rendering: bool,
 }

impl Default for IFS{
    fn default() -> Self {
        Self {
            title: String::from("Untitled"),
            width: 512,
            height: 512,
            brightness: 1.0,
            gamma_inv: 1.0,
            gamma_thresh: 0.0,
            vibrancy: 1.0,
            background_color: [0.0, 0.0, 0.0],
            fov: 60.0,
            aperture: 0.0,
            fdist: 10.0,
            dof: 0.25,
            entropy: 0.01,
            fuse: 20,
            pause_rendering: false,
        }
    }
}