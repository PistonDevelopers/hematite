
//! A 3D camera.

use vecmath::{
    Vector3,
    Matrix4,
    vec3_normalized_sub,
    vec3_cross,
    vec3_dot
};

use std::f32::consts::PI;

pub struct Camera {
    pub position: Vector3,
    pub target: Vector3,
    pub right: Vector3,
    pub up: Vector3,
}

pub struct CameraSettings {
    pub fov: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    /// Computes the direction forward.
    ///
    /// Returns the normalized difference between target and position.
    pub fn forward(&self) -> Vector3 {
        vec3_normalized_sub(self.position, self.target)
    }

    /// Computes an orthogonal matrix for the camera.
    ///
    /// This matrix can be used to transform coordinates to the screen.
    pub fn orthogonal(&self) -> Matrix4 {
        let p = self.position;
        let r = self.right;
        let u = self.up;
        let f = self.forward();
        [
            [r[0], u[0], f[0], 0.0],
            [r[1], u[1], f[1], 0.0],
            [r[2], u[2], f[2], 0.0],
            [-vec3_dot(r, p), -vec3_dot(u, p), -vec3_dot(f, p), 1.0]
        ]
    }

    pub fn update_right(&mut self) {
        self.right = vec3_cross(self.up, self.forward());
    }
}

impl CameraSettings {
    /// Computes a projection matrix for the camera settings.
    pub fn projection(&self) -> Matrix4 {
        let f = 1.0 / (self.fov * (PI / 360.0)).tan();
        let (far, near) = (self.far_clip, self.near_clip);
        [
            [f / self.aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0]
        ]
    }
}

