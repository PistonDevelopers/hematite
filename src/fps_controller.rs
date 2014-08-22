use std::f32::consts::{PI, SQRT2};
use piston::input::{KeyPress, KeyRelease, MouseRelativeMove};
use piston::{GameEvent, Input, Update};
use cam::Camera;

bitflags!(flags Keys: u8 {
    static MoveForward = 0b00000001,
    static MoveBack    = 0b00000010,
    static StrafeLeft  = 0b00000100,
    static StrafeRight = 0b00001000,
    static FlyUp       = 0b00010000,
    static FlyDown     = 0b00100000
})

pub struct FPSController {
    pub yaw: f32,
    pub pitch: f32,
    pub keys: Keys,
    pub direction: [f32, ..3],
    pub velocity: f32,
}

impl FPSController {
    pub fn new() -> FPSController {
        FPSController {
            yaw: 0.0,
            pitch: 0.0,
            keys: Keys::empty(),
            direction: [0.0, 0.0, 0.0],
            velocity: 1.0,
        }
    }

    pub fn event(&mut self, e: &GameEvent, camera: &mut Camera) {
        let &FPSController {
            yaw: ref mut yaw,
            pitch: ref mut pitch,
            keys: ref mut keys,
            direction: ref mut direction,
            velocity: ref mut velocity,
        } = self;

        match *e {
            Update(args) => {
                let dt = args.dt as f32;
                let dh = dt * *velocity * 8.0;
                let [dx, dy, dz] = *direction;
                let (s, c) = (yaw.sin(), yaw.cos());
                camera.position[0] += (s * dx - c * dz) * dh;
                camera.position[1] += dy * dt * 4.0;
                camera.position[2] += (s * dz + c * dx) * dh;
            }
            Input(MouseRelativeMove { dx, dy, .. }) => {
                *yaw = (*yaw - dx as f32 / 360.0 * PI / 4.0) % (2.0 * PI);
                *pitch += dy as f32 / 360.0 * PI / 4.0;
                *pitch = (*pitch).min(PI / 2.0).max(-PI / 2.0);
                camera.set_yaw_pitch(*yaw, *pitch);
            }
            Input(KeyPress { key }) => {
                use piston::input::keyboard::{A, D, S, W, Space, LShift, LCtrl};
                let [dx, dy, dz] = *direction;
                let sgn = |x: f32| if x == 0.0 {0.0} else {x.signum()};
                let set = |k, x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / SQRT2, z / SQRT2)
                    } else {
                        (x, z)
                    };
                    *direction = [x, y, z];
                    keys.insert(k);
                };
                match key {
                    W => set(MoveForward, -1.0, dy, dz),
                    S => set(MoveBack, 1.0, dy, dz),
                    A => set(StrafeLeft, dx, dy, 1.0),
                    D => set(StrafeRight, dx, dy, -1.0),
                    Space => set(FlyUp, dx, 1.0, dz),
                    LShift => set(FlyDown, dx, -1.0, dz),
                    LCtrl => *velocity = 2.0,
                    _ => {}
                }
            }
            Input(KeyRelease { key }) => {
                use piston::input::keyboard::{A, D, S, W, Space, LShift, LCtrl};
                let [dx, dy, dz] = *direction;
                let sgn = |x: f32| if x == 0.0 {0.0} else {x.signum()};
                let set = |x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / SQRT2, z / SQRT2)
                    } else {
                        (x, z)
                    };
                    *direction = [x, y, z];
                };
                let release = |key, rev_key, rev_val| {
                    keys.remove(key);
                    if keys.contains(rev_key) {rev_val} else {0.0}
                };
                match key {
                    W => set(release(MoveForward, MoveBack, 1.0), dy, dz),
                    S => set(release(MoveBack, MoveForward, -1.0), dy, dz),
                    A => set(dx, dy, release(StrafeLeft, StrafeRight, -1.0)),
                    D => set(dx, dy, release(StrafeRight, StrafeLeft, 1.0)),
                    Space => set(dx, release(FlyUp, FlyDown, -1.0), dz),
                    LShift => set(dx, release(FlyDown, FlyUp, 1.0), dz),
                    LCtrl => *velocity = 1.0,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
