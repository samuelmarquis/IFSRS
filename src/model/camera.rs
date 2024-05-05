use std::hash::{Hash, Hasher};
use crate::rendering::gpu_structs::CameraStruct;
use nalgebra::{Matrix4, Vector3, Quaternion, Point3, Vector4, convert, Rotation3};
use serde::{Deserialize, Serialize};
use crate::util::math_extensions::{transform_vector, to_radians};
#[derive(Serialize, Deserialize)]
pub enum ProjectionType {
    Perspective
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Camera {
    pub position: Point3<f64>,
    pub orientation: Quaternion<f64>,
    pub right_direction: Vector3<f64>,
    pub up_direction: Vector3<f64>,
    pub forward_direction: Vector3<f64>,

    pub fov: f64,
    pub aperture: f64,
    pub focus_distance: f64,
    pub dof: f64,

    //pub projection_type: ProjectionType,
}

impl Hash for Camera {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            std::mem::transmute::<Point3<f64>, [u64; 3]>(self.position).hash(state);
            std::mem::transmute::<Quaternion<f64>, [u64; 4]>(self.orientation).hash(state);
            std::mem::transmute::<f64, u64>(self.fov).hash(state);
            std::mem::transmute::<f64, u64>(self.aperture).hash(state);
            std::mem::transmute::<f64, u64>(self.focus_distance).hash(state);
            std::mem::transmute::<f64, u64>(self.dof).hash(state);
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::identity(),

            right_direction: Vector3::new(1.0, 0.0, 0.0),
            up_direction: Vector3::new(0.0, 1.0, 0.0),
            forward_direction: Vector3::new(0.0, 0.0, 1.0),


            fov: 60.0,
            aperture: 0.0,
            focus_distance: 10.0,
            dof: 0.25,
            // projection_type: ProjectionType::Perspective,
        }
    }
}

impl Camera {
    fn get_view_projection_matrix(&self, ) -> [[f32;4];4] {
        let view = Matrix4::look_at_rh(&self.position, &(self.position - self.forward_direction), &(-self.up_direction));
        let projection = Matrix4::new_perspective(1.0, to_radians(1.0+(self.fov % 179.0)), 0.001, f64::MAX);
        println!("{:?}\n{:?}\n", view, projection);
        (view * projection).map(|e| e as f32).into()
    }



    fn update_direction_vectors(&mut self){
        //     RightDirection = Vector3.Transform(new Vector3(1.0f, 0.0f, 0.0f), Orientation);
        //     UpDirection = Vector3.Transform(new Vector3(0.0f, 1.0f, 0.0f), Orientation);
        //     ForwardDirection = Vector3.Transform(new Vector3(0.0f, 0.0f, 1.0f), Orientation);
        self.right_direction = transform_vector(&Vector3::new(1.0, 0.0, 0.0), &self.orientation);
        self.up_direction = transform_vector(&Vector3::new(0.0, 1.0, 0.0), &self.orientation);
        self.forward_direction = transform_vector(&Vector3::new(0.0, 0.0, 1.0), &self.orientation);

        // self.up_direction = nalgebra::Transform3

        // // self.orientation
        // let rotation = Rotation3::from_(&self.orientation);
        // let transformed = rotation.transform_vector(&nalgebra::Vector3::<f64>::new(1.0, 0.0, 0.0));
        // // nalgebra::Vector3<f64>::new(1.0, 0.0, 0.0).transform_vector(self.orientation)
    }

    pub fn translate(&mut self, translate_vector: Vector3<f64>){
        self.position += self.right_direction * translate_vector.x
                      + self.up_direction * translate_vector.y
                      + self.forward_direction * translate_vector.z;
    }

    pub fn create_camera_struct(&mut self) -> CameraStruct {
        self.update_direction_vectors();

        let pos: Vector4<f32> = convert(self.position.coords.push(1.0));
        let forward: Vector4<f32> = convert(self.forward_direction.push(1.0));
        let focus: Vector4<f32> = convert((self.position + self.focus_distance * self.forward_direction).coords.push(0.0));

        CameraStruct {
            view_proj_mat: self.get_view_projection_matrix(),
            position: pos.into(),
            forward: forward.into(),
            focus_point: focus.into(),
            aperture: self.aperture as f32,
            focus_distance: self.focus_distance as f32,
            depth_of_field: self.dof as f32,
            projection_type: 0, // TODO: never forget 🦅🦅🎇🎆
        }
    }
}
