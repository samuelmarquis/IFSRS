use nalgebra::{Quaternion, Vector3};
use std::f64::consts::PI;
pub fn transform_vector(val: &Vector3<f64>, rot: &Quaternion<f64>) -> Vector3<f64>{
    let x2 = rot[0] + rot[0];
    let y2 = rot[1] + rot[1];
    let z2 = rot[2] + rot[2];

    let wx2 = rot[3] * x2;
    let wy2 = rot[3] * y2;
    let wz2 = rot[3] * z2;
    let xx2 = rot[0] * x2;
    let xy2 = rot[0] * y2;
    let xz2 = rot[0] * z2;
    let yy2 = rot[1] * y2;
    let yz2 = rot[1] * z2;
    let zz2 = rot[2] * z2;

    Vector3::new(
        val.x * (1.0 - yy2 - zz2) + val.y * (xy2 - wz2) + val.z * (xz2 + wy2),
        val.x * (xy2 + wz2) + val.y * (1.0 - xx2 - zz2) + val.z * (yz2 - wx2),
        val.x * (xz2 - wy2) + val.y * (yz2 + wx2) + val.z * (1.0 - xx2 - yy2)
    )
}

pub fn to_radians(val: f64) -> f64 {
    (PI / 180.0) * val
}