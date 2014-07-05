
//! A 3D camera.

pub type Vector3 = [f64, ..3];

pub struct Camera {
    pub position: Vector3,
    pub target: Vector3,
    pub right: Vector3,
    pub up: Vector3,
}


