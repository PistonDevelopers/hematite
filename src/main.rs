#![feature(globs)]

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
    GameWindowSettings,
    Render,
    Update,
};

use cam::{Camera, CameraSettings};
use texture::Texture;

pub mod shader;
pub mod cube;
pub mod cam;
pub mod texture;
pub mod vecmath;

fn main() {
    let mut window = Window::new(
        GameWindowSettings {
            title: "Hematite".to_string(),
            size: [600, 600], // [640, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );

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

    let mut camera = Camera {
        position: [0.0, 0.0, 0.0],
        target: [0.0, 0.0, 0.0],
        right: [1.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0]
    };
    let mut t: f32 = 0.0;
    let radius = 4.0;
    let speed = 1.0;

    for e in GameIterator::new(&mut window, &game_iter_settings) {
        match e {
            Render(_args) => {
                // TEST
                // println!("Seconds: {}", t);

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
                    tri.push([v[1], v[2], v[3]]);
                }
                buffer.load_data(tri.as_slice());

                camera.position = [
                    (speed * t).cos() * radius,
                    0.5,
                    (speed * t).sin() * radius
                ];
                camera.update_right();
                // TEST
                // println!("Camera: {:?}", camera);

                shader.set_view(camera.orthogonal());
                shader.bind();
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                shader.render(&buffer);
            },
            Update(args) => {
                t += args.dt as f32;
            },
            _ => {}
        }
    }
}
