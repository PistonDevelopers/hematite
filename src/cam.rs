
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
    pub up: Vector3,
    pub right: Vector3,
    pub forward: Vector3
}

pub struct CameraSettings {
    pub fov: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub aspect_ratio: f32,
}

impl Camera {
    /// Constructs a new camera.
    ///
    /// Places the camera at [x, y, z], looking towards pozitive z.
    pub fn new(x: f32, y: f32, z: f32) -> Camera {
        Camera {
            position: [x, y, z],
            up: [0.0, 1.0, 0.0],
            right: [1.0, 0.0, 0.0],
            forward: [0.0, 0.0, 1.0]
        }
    }

    /// Computes an orthogonal matrix for the camera.
    ///
    /// This matrix can be used to transform coordinates to the screen.
    pub fn orthogonal(&self) -> Matrix4 {
        let p = self.position;
        let r = self.right;
        let u = self.up;
        let f = self.forward;
        [
            [r[0], u[0], f[0], 0.0],
            [r[1], u[1], f[1], 0.0],
            [r[2], u[2], f[2], 0.0],
            [-vec3_dot(r, p), -vec3_dot(u, p), -vec3_dot(f, p), 1.0]
        ]
    }

    pub fn look_at(&mut self, x: f32, y: f32, z: f32) {
        self.forward = vec3_normalized_sub(self.position, [x, y, z]);
        self.update_right();
    }

    pub fn set_yaw_pitch(&mut self, yaw: f32, pitch: f32) {
        let (y_s, y_c, p_s, p_c) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
        self.forward = [y_s * p_c, p_s, y_c * p_c];
        self.up = [y_s * -p_s, p_c, y_c * -p_s];
        self.update_right();
    }

    fn update_right(&mut self) {
        self.right = vec3_cross(self.up, self.forward);
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

