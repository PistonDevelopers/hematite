#![feature(globs, macro_rules)]

extern crate debug;
extern crate piston;
extern crate sdl2_game_window;
extern crate gl;
extern crate hgl;
extern crate image;
extern crate libc;
extern crate cgmath;

use Window = sdl2_game_window::GameWindowSDL2;
use piston::{
    AssetStore,
    GameIterator,
    GameIteratorSettings,
    GameWindow,
    GameWindowSettings,
    KeyPress,
    KeyRelease,
    MouseRelativeMove,
    Render,
    Update
};

use cam::{Camera, CameraSettings};
use texture::Texture;

use std::f32::consts::{PI, SQRT2};

pub mod shader;
pub mod cube;
pub mod cam;
pub mod texture;
pub mod vecmath;

bitflags!(flags Keys: u8 {
    static MoveForward = 0b00000001,
    static MoveBack    = 0b00000010,
    static StrafeLeft  = 0b00000100,
    static StrafeRight = 0b00001000,
    static FlyUp       = 0b00010000,
    static FlyDown     = 0b00100000
})

fn main() {
    let mut window = Window::new(
        GameWindowSettings {
            title: "Hematite".to_string(),
            size: [600, 600], // [640, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );

    window.capture_cursor(true);

    let asset_store = AssetStore::from_folder("assets");

    // Load texture.
    let texture = asset_store.path("minecraft-texture.png").unwrap();
    let texture = Texture::from_path(&texture).unwrap();

    let game_iter_settings = GameIteratorSettings {
        updates_per_second: 120,
        max_frames_per_second: 60,
    };

    let shader = shader::Shader::new();
    let mut buffer = shader.new_buffer();

    shader.set_projection(CameraSettings {
        fov: 90.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: 1.0
    }.projection());

    let mut camera = Camera::new(0.5, 0.5, 4.0);

    let mut yaw = 0.0;
    let mut pitch = 0.0;
    camera.set_yaw_pitch(yaw, pitch);

    let mut keys = Keys::empty();
    let mut direction = [0.0, 0.0, 0.0];
    let mut velocity = 1.0;

    for e in GameIterator::new(&mut window, &game_iter_settings) {
        match e {
            Render(_args) => {
                let mut tri: Vec<[([f32, ..3], [f32, ..2], [f32, ..3]), ..3]> = vec![];
                for face in cube::FaceIterator::new() {
                    let v = face.vertices(0.0, 0.0, 0.0);
                    let (tx, ty) = texture::Grass.get_src_xy();
                    let t = texture.square(tx, ty);
                    let v = [
                        (v[0], t[3], [1.0, 0.0, 0.0]),
                        (v[1], t[2], [0.0, 1.0, 0.0]),
                        (v[2], t[1], [0.0, 0.0, 1.0]),
                        (v[3], t[0], [1.0, 0.0, 1.0])
                    ];
                    tri.push([v[0], v[1], v[2]]);
                    tri.push([v[1], v[3], v[2]]);
                }
                buffer.load_data(tri.as_slice());

                shader.set_view(camera.orthogonal());
                shader.bind();
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                shader.render(&buffer);
            },
            Update(args) => {
                let dt = args.dt as f32;
                let dh = dt * velocity * 3.0;
                let [dx, dy, dz] = direction;
                let (s, c) = (yaw.sin(), yaw.cos());
                camera.position[0] += (s * dx - c * dz) * dh;
                camera.position[1] += dy * dt * 4.0;
                camera.position[2] += (s * dz + c * dx) * dh;
            },
            MouseRelativeMove(args) => {
                yaw = (yaw - args.dx as f32 / 360.0 * PI / 4.0) % (2.0 * PI);
                pitch += args.dy as f32 / 360.0 * PI / 4.0;
                pitch = pitch.min(PI / 2.0).max(-PI / 2.0);
                camera.set_yaw_pitch(yaw, pitch);
            }
            KeyPress(args) => {
                use piston::keyboard::{A, D, S, W, Space, LShift, LCtrl};
                let [dx, dy, dz] = direction;
                let sgn = |x: f32| if x == 0.0 {0.0} else {x.signum()};
                let set = |k, x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / SQRT2, z / SQRT2)
                    } else {
                        (x, z)
                    };
                    direction = [x, y, z];
                    keys.insert(k);
                };
                match args.key {
                    W => set(MoveForward, -1.0, dy, dz),
                    S => set(MoveBack, 1.0, dy, dz),
                    A => set(StrafeLeft, dx, dy, 1.0),
                    D => set(StrafeRight, dx, dy, -1.0),
                    Space => set(FlyUp, dx, 1.0, dz),
                    LShift => set(FlyDown, dx, -1.0, dz),
                    LCtrl => velocity = 2.0,
                    _ => {}
                }
            }
            KeyRelease(args) => {
                use piston::keyboard::{A, D, S, W, Space, LShift, LCtrl};
                let [dx, dy, dz] = direction;
                let sgn = |x: f32| if x == 0.0 {0.0} else {x.signum()};
                let set = |x: f32, y: f32, z: f32| {
                    let (x, z) = (sgn(x), sgn(z));
                    let (x, z) = if x != 0.0 && z != 0.0 {
                        (x / SQRT2, z / SQRT2)
                    } else {
                        (x, z)
                    };
                    direction = [x, y, z];
                };
                let release = |key, rev_key, rev_val| {
                    keys.remove(key);
                    if keys.contains(rev_key) {rev_val} else {0.0}
                };
                match args.key {
                    W => set(release(MoveForward, MoveBack, 1.0), dy, dz),
                    S => set(release(MoveBack, MoveForward, -1.0), dy, dz),
                    A => set(dx, dy, release(StrafeLeft, StrafeRight, -1.0)),
                    D => set(dx, dy, release(StrafeRight, StrafeLeft, 1.0)),
                    Space => set(dx, release(FlyUp, FlyDown, -1.0), dz),
                    LShift => set(dx, release(FlyDown, FlyUp, 1.0), dz),
                    LCtrl => velocity = 1.0,
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
