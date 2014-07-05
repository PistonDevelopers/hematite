
//! A 3D camera.

use vecmath::{
    Vector3,
    Matrix4x3,
};

pub struct Camera {
    pub position: Vector3,
    pub target: Vector3,
    pub right: Vector3,
    pub up: Vector3,
}

pub struct CameraSettings {
    pub fov_rad: f64,
    pub near_clip: f64,
    pub far_clip: f64,
    pub aspect_ratio: f64,
}

impl Camera {
    /// Computes the direction forward.
    ///
    /// Returns the normalized difference between target and position.
    pub fn forward(&self) -> Vector3 {
        let diff = [
                self.target[0] - self.position[0],
                self.target[1] - self.position[1],
                self.target[2] - self.position[2],
            ];
        let len = diff[0] * diff[0] + diff[1] * diff[1] + diff[2] * diff[2];
        [
            diff[0] / len,
            diff[1] / len,
            diff[2] / len,
        ]
    }

    /// Computes an orthogonal matrix for the camera.
    ///
    /// This matrix can be used to transform coordinates to the screen.
    pub fn orthogonal(&self) -> Matrix4x3 {
        [
            self.right,
            self.up,
            self.forward(),
            self.position
        ]
    }
}

