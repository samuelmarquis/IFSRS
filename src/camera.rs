use crate::gpu_structs::CameraStruct;
use nalgebra::{Matrix4, Vector3, Quaternion, Point3, Vector4, convert};

pub enum ProjectionType {
    Perspective
}

pub struct Camera {
    pub position: Point3<f64>,
    pub orientation: Quaternion<f64>,
    pub right_direction: Vector3<f64>,
    pub up_direction: Vector3<f64>,
    pub forward_direction: Vector3<f64>,

    pub fov: f64,
    pub aperture: f64,
    pub focus_disance: f64,
    pub depth_of_field: f64,

    pub projection_type: ProjectionType,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, -10.0),
            orientation: Quaternion::identity(),

            right_direction: Vector3::new(1.0, 0.0, 0.0),
            up_direction: Vector3::new(0.0, 1.0, 0.0),
            forward_direction: Vector3::new(0.0, 0.0, 1.0),

            fov: 60.0,
            aperture: 0.0,
            focus_disance: 10.0,
            depth_of_field: 0.25,
            projection_type: ProjectionType::Perspective,
        }
    }
}

impl Camera {
    fn get_view_projection_matrix(&self, ) -> [[f32;4];4] {
        let view = Matrix4::look_at_lh(&self.position, &(self.position - self.forward_direction), &self.up_direction);
        let projection = Matrix4::new_perspective(1.0, self.fov, 0.001, f64::INFINITY);
        (view * projection).map(|e| e as f32).into()
    }

    pub fn create_camera_struct(&self) -> CameraStruct {
        let pos: Vector4<f32> = convert(self.position.coords.push(1.0));
        let forward: Vector4<f32> = convert(self.forward_direction.push(1.0));
        let focus: Vector4<f32> = convert((self.position + self.focus_disance * self.forward_direction).coords.push(0.0));

        CameraStruct {
            view_proj_mat: self.get_view_projection_matrix(),
            position: pos.into(),
            forward: forward.into(),
            focus_point: focus.into(),
            aperture: self.aperture as f32,
            focus_distance: self.focus_disance as f32,
            depth_of_field: self.depth_of_field as f32,
            projection_type: 0, // TODO: never forget 🦅🦅🎇🎆
        }
    }
}