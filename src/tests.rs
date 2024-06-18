use crate::model::camera::Camera;

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Matrix4, Point3, Quaternion, Rotation3, Unit, UnitQuaternion, Vector3};
    use std::f32;
    use f32::INFINITY;

    #[test]
    fn test_get_view_projection_matrix() {
        let camera = Camera::default();
        let view_projection_matrix = camera.get_view_projection_matrix();

        // Check that the matrix is not empty
        assert_ne!(view_projection_matrix, [[0.0; 4]; 4]);

        // Check that the matrix is a 4x4 matrix
        assert_eq!(view_projection_matrix.len(), 4);
        for row in view_projection_matrix {
            assert_eq!(row.len(), 4);
        }

        // Check that the matrix is not NaN
        for row in view_projection_matrix {
            for element in row {
                assert!(!element.is_nan());
            }
        }

        // Check that the matrix is not infinite
        for row in view_projection_matrix {
            for element in row {
                assert!(!element.is_infinite());
            }
        }

        // Check that the matrix is not degenerate
        let matrix = Matrix4::from(view_projection_matrix);
        assert_ne!(matrix.determinant(), 0.0);

        // Check that the matrix is a valid view-projection matrix
        // A view-projection matrix should transform the camera position to the origin
        let camera_position = camera.position.cast::<f32>();
        let transformed_position = matrix.transform_point(&camera_position);
        assert!(transformed_position.z < 0.0 && transformed_position.z > -1.0);

        // A view-projection matrix should transform the camera forward direction to the negative Z axis
        let camera_forward = camera.forward_direction.cast::<f32>();
        let transformed_forward = matrix.transform_vector(&camera_forward);
        assert!(transformed_forward.z > 0.0);
        assert!(transformed_forward.x.abs() < 0.01);
        assert!(transformed_forward.y.abs() < 0.01);
    }
    #[test]
    fn test_view_projection_matrix() {
        let camera = Camera::default();
        let view_projection_matrix = camera.get_view_projection_matrix();
        let matrix = Matrix4::from(view_projection_matrix);

        // A perspective projection matrix should map the camera's near plane to the cube with corners (-1,-1,-1) and (1,1,-1)
        let near_point = Point3::new(0.0, 0.0, -0.001);
        let transformed_near_point = matrix.transform_point(&near_point);
        assert!(transformed_near_point.x >= -1.0 && transformed_near_point.x <= 1.0);
        assert!(transformed_near_point.y >= -1.0 && transformed_near_point.y <= 1.0);
        assert!(transformed_near_point.z >= -1.0 && transformed_near_point.z <= -1.0);

        // A perspective projection matrix should map the camera's far plane to the cube with corners (-1,-1,1) and (1,1,1)
        let far_point = Point3::new(0.0, 0.0, -f32::MAX);
        let transformed_far_point = matrix.transform_point(&far_point);
        assert!(transformed_far_point.x >= -1.0 && transformed_far_point.x <= 1.0);
        assert!(transformed_far_point.y >= -1.0 && transformed_far_point.y <= 1.0);
        assert!(transformed_far_point.z >= 1.0 && transformed_far_point.z <= 1.0);

        // A view matrix should rotate the world so that the camera's up direction is the y-axis
        let camera_up = camera.up_direction.cast::<f32>();
        let transformed_up = matrix.transform_vector(&camera_up);
        assert!(transformed_up.x.abs() < 0.01);
        assert!(transformed_up.y < 0.0);
        assert!(transformed_up.z.abs() < 0.01);

        // A view matrix should rotate the world so that the camera's right direction is the x-axis
        let camera_right = camera.right_direction.cast::<f32>();
        let transformed_right = matrix.transform_vector(&camera_right);
        assert!(transformed_right.x < 0.0);
        assert!(transformed_right.y.abs() < 0.01);
        assert!(transformed_right.z.abs() < 0.01);
    }
    #[test]
    fn test_update_direction_vectors() {
        let mut camera = Camera::default();
        let axis = Vector3::new(0.0, 1.0, 0.0).normalize();
        let quaternion = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), std::f64::consts::PI / 2.0).into_inner();
        camera.orientation = quaternion;
        camera.update_direction_vectors();
        assert!(camera.right_direction.x.abs() < 0.01);
        assert!(camera.right_direction.y.abs() < 0.01);
        assert!(camera.right_direction.z < 0.0);
        assert!(camera.up_direction.x.abs() < 0.01);
        assert!(camera.up_direction.y > 0.0);
        assert!(camera.up_direction.z.abs() < 0.01);
        assert!(camera.forward_direction.x > 0.0);
        assert!(camera.forward_direction.y.abs() < 0.01);
        assert!(camera.forward_direction.z.abs() < 0.01);
    }
    #[test]
    fn test_translate() {
        let mut camera = Camera::default();
        camera.translate(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(camera.position, Point3::new(1.0, 2.0, 3.0));
    }
    #[test]
    fn test_create_camera_struct() {
        let mut camera = Camera::default();
        let camera_struct = camera.create_camera_struct();

        assert_eq!(camera_struct.position[3], 1.0);
        assert_eq!(camera_struct.forward[3], 1.0);
        assert_eq!(camera_struct.focus_point[3], 0.0);
        assert_eq!(camera_struct.aperture, camera.aperture as f32);
        assert_eq!(camera_struct.focus_distance, camera.focus_distance as f32);
        assert_eq!(camera_struct.depth_of_field, camera.dof as f32);
        assert_eq!(camera_struct.projection_type, 0);
    }
    #[test]
    fn test_camera_interactions() {
        let mut camera = Camera::default();

        // Define the rotations to test
        let rotations = vec![
            (Vector3::new(0.0, 1.0, 0.0), std::f64::consts::PI / 2.0), // 90 degrees around Y-axis
            (Vector3::new(0.0, 1.0, 0.0), -std::f64::consts::PI / 2.0), // -90 degrees around Y-axis
            (Vector3::new(1.0, 0.0, 0.0), std::f64::consts::PI / 2.0), // 90 degrees around X-axis
            (Vector3::new(1.0, 0.0, 0.0), -std::f64::consts::PI / 2.0), // -90 degrees around X-axis
            (Vector3::new(0.0, 0.0, 1.0), std::f64::consts::PI / 2.0), // 90 degrees around Z-axis
            (Vector3::new(0.0, 0.0, 1.0), -std::f64::consts::PI / 2.0), // -90 degrees around Z-axis
        ];

        // Define the translations to test
        let translations = vec![
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, -1.0),
        ];

        for (axis, angle) in rotations {
            for translation in &translations {
                camera = Camera::default();
                camera.translate(*translation);
                camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), angle).into_inner();
                camera.update_direction_vectors();
                camera.translate(*translation);
                let camera_struct = camera.create_camera_struct();

                // Calculate the expected position and orientation
                let rotation_matrix = UnitQuaternion::from_axis_angle(&Unit::new_normalize(axis), angle).to_rotation_matrix();
                let expected_position = rotation_matrix * translation + translation;
                let expected_forward = rotation_matrix * Vector3::new(0.0, 0.0, 1.0);

                assert!((camera_struct.position[0] as f64 - expected_position.x).abs() < 0.001);
                assert!((camera_struct.position[1] as f64- expected_position.y).abs() < 0.001);
                assert!((camera_struct.position[2] as f64- expected_position.z).abs() < 0.001);
                assert!((camera_struct.forward[0] as f64- expected_forward.x).abs() < 0.001);
                assert!((camera_struct.forward[1] as f64- expected_forward.y).abs() < 0.001);
                assert!((camera_struct.forward[2] as f64 - expected_forward.z).abs() < 0.001);
            }
        }
    }
    #[test]
    fn test_camera_translation_and_orientation() {
        let mut camera = Camera::default();
        camera.translate(Vector3::new(1.0, 2.0, 3.0));
        let rotation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), std::f64::consts::PI / 2.0);
        camera.orientation = rotation.into_inner();
        camera.update_direction_vectors();

        let camera_struct = camera.create_camera_struct();
        assert_eq!(camera_struct.position[0], 1.0);
        assert_eq!(camera_struct.position[1], 2.0);
        assert_eq!(camera_struct.position[2], 3.0);
        assert_eq!(camera_struct.forward[0], 1.0);
        assert_eq!(camera_struct.forward[1], 0.0);
        assert!((camera_struct.forward[2] - 0.0).abs() < 0.001);
    }
    #[test]
    fn test_camera_rotation_and_projection() {
        let mut camera = Camera::default();
        let rotation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), std::f64::consts::PI / 2.0);
        camera.orientation = rotation.into_inner();
        camera.update_direction_vectors();

        let camera_struct = camera.create_camera_struct();
        let view_proj_mat = camera_struct.view_proj_mat;
        let far_point = Point3::new(0.0, 0.0, -f32::MAX);
        let transformed_far_point = Matrix4::from(view_proj_mat).transform_point(&far_point);

        dbg!(transformed_far_point);

        assert_eq!(transformed_far_point.x, f32::INFINITY);
        assert!((transformed_far_point.y - 0.0).abs() < 0.001);
        assert!(transformed_far_point.z >= 0.99 && transformed_far_point.z <= 1.01);
    }
    #[test]
    fn test_repeated_rotations() {
        let mut camera = Camera::default();
        dbg!(camera.forward_direction);
        camera.orientation = UnitQuaternion::from_axis_angle(
            &Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)),
            std::f64::consts::PI / 2.0).into_inner();
        camera.update_direction_vectors();
        dbg!(camera.forward_direction);
        camera.orientation = UnitQuaternion::from_axis_angle(
            &Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)),
            std::f64::consts::PI / 2.0).into_inner();
        camera.update_direction_vectors();
        let camera_struct = camera.create_camera_struct();
        dbg!(camera.forward_direction);
        assert!((camera_struct.forward[0] - 1.0).abs() < 0.001);
        assert!((camera_struct.forward[1] - 0.0).abs() < 0.001);
        assert!((camera_struct.forward[2] - 0.0).abs() < 0.001);
    }
    #[test]
    fn test_repeated_translations() {
        let mut camera = Camera::default();
        camera.translate(Vector3::new(1.0, 0.0, 0.0));
        camera.translate(Vector3::new(0.0, 1.0, 0.0));
        let camera_struct = camera.create_camera_struct();
        assert!((camera_struct.position[0] - 1.0).abs() < 0.001);
        assert!((camera_struct.position[1] - 1.0).abs() < 0.001);
        assert!((camera_struct.position[2] - 0.0).abs() < 0.001);
    }
    #[test]
    fn test_rotation_translation() {
        let mut camera = Camera::default();
        camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), std::f64::consts::PI / 2.0).into_inner();
        camera.update_direction_vectors();
        camera.translate(Vector3::new(1.0, 0.0, 0.0));
        let camera_struct = camera.create_camera_struct();
        dbg!(camera_struct.position);
        assert!((camera_struct.position[0] - 0.0).abs() < 0.001);
        assert!((camera_struct.position[1] - 0.0).abs() < 0.001);
        assert!((camera_struct.position[2] - -1.0).abs() < 0.001);
    }
    #[test]
    fn test_composition_of_rotations() {
        let mut camera = Camera::default();

        camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)), std::f64::consts::PI / 2.0).into_inner();
        camera.update_direction_vectors();
        //camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 1.0, 0.0)), std::f64::consts::PI / 2.0).into_inner();
        //camera.update_direction_vectors();
        camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)), 3.0 * std::f64::consts::PI / 2.0).into_inner();
        camera.update_direction_vectors();
        let camera_struct = camera.create_camera_struct();
        let mut camera2 = Camera::default();
        camera2.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(0.0, 0.0, 1.0)), std::f64::consts::PI / 2.0).into_inner();
        camera2.update_direction_vectors();
        let camera_struct2 = camera2.create_camera_struct();
        dbg!(camera_struct.forward);
        dbg!(camera_struct2.forward);
        assert!((camera_struct.forward[0] - camera_struct2.forward[0]).abs() < 0.001);
        assert!((camera_struct.forward[1] - camera_struct2.forward[1]).abs() < 0.001);
        assert!((camera_struct.forward[2] - camera_struct2.forward[2]).abs() < 0.001);
    }
    #[test]
    fn test_edge_cases() {
        let mut camera = Camera::default();
        camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)), 0.0).into_inner();
        camera.update_direction_vectors();
        let camera_struct = camera.create_camera_struct();
        assert!((camera_struct.forward[0] - 0.0).abs() < 0.001);
        assert!((camera_struct.forward[1] - 0.0).abs() < 0.001);
        assert!((camera_struct.forward[2] - 1.0).abs() < 0.001);
    }
    #[test]
    fn test_large_rotations() {
        let mut camera = Camera::default();
        camera.orientation = UnitQuaternion::from_axis_angle(&Unit::new_normalize(Vector3::new(1.0, 0.0, 0.0)), 2.0 * std::f64::consts::PI).into_inner();
        camera.update_direction_vectors();
        let camera_struct = camera.create_camera_struct();
        assert!((camera_struct.forward[0] - 0.0).abs() < 0.001);
        assert!((camera_struct.forward[1] - 0.0).abs() < 0.001);
        assert!((camera_struct.forward[2] - 1.0).abs() < 0.001);
    }
    #[test]
    fn test_small_translations() {
        let mut camera = Camera::default();
        camera.translate(Vector3::new(0.01, 0.01, 0.01));
        let camera_struct = camera.create_camera_struct();
        assert!((camera_struct.position[0] - 0.01).abs() < 0.001);
        assert!((camera_struct.position[1] - 0.01).abs() < 0.001);
        assert!((camera_struct.position[2] - 0.01).abs() < 0.001);
    }
}