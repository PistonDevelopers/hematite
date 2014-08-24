#![feature(globs, macro_rules, phase)]

extern crate debug;
extern crate piston;
extern crate sdl2;
extern crate sdl2_game_window;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;
extern crate libc;
extern crate cgmath;
extern crate time;

use sdl2_game_window::GameWindowSDL2 as Window;
use piston::input;
use piston::cam;
use piston::{
    AssetStore,
    GameIterator,
    GameIteratorSettings,
    GameWindow,
    GameWindowSettings,
    Input,
    Render,
    Update,
};

use array::*;
use texture::Texture;

pub mod array;
pub mod cube;
pub mod fps_counter;
pub mod shader;
pub mod texture;
pub mod vecmath;

fn main() {
    let mut window = Window::new(
        piston::shader_version::opengl::OpenGL_3_3,
        GameWindowSettings {
            title: "Hematite".to_string(),
            size: [854, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );
    let (mut device, frame) = window.gfx();

    let asset_store = AssetStore::from_folder("../assets");

    // Load texture.
    let texture = asset_store.path("texture.png").unwrap();
    let texture = Texture::from_path(&texture, &mut device).unwrap();

    let game_iter_settings = GameIteratorSettings {
        updates_per_second: 120,
        max_frames_per_second: 10000,
    };

    let mut renderer = shader::Renderer::new(device, frame, texture.tex);

    renderer.set_projection(cam::CameraPerspective {
        fov: 70.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: {
            let (w, h) = window.get_size();
            (w as f32) / (h as f32)
        }
    }.projection());

    let mut first_person_settings = cam::FirstPersonSettings::default();
    first_person_settings.speed_horizontal = 8.0;
    first_person_settings.speed_vertical = 4.0;
    let mut first_person = cam::FirstPerson::new(
        0.5, 0.5, 4.0,
        first_person_settings
    );

    // Disable V-Sync.
    sdl2::video::gl_set_swap_interval(0);

    let mut fps_counter = fps_counter::FPSCounter::new();

    let mut capture_cursor = false;
    println!("Press C to capture mouse");
    let mut extrapolate_time = false;
    println!("Press X to extrapolate time");

    let buffer = {
        let mut tri = vec![];
        for face in cube::FaceIterator::new() {
            let xyz = face.vertices([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
            let [u0, v1, u1, v0] = [0.0, 0.75, 0.25, 1.0];
            let uv = [
                [u1, v0],
                [u0, v0],
                [u0, v1],
                [u1, v1]
            ];
            let v = [
                (xyz[0], uv[0], [1.0, 0.0, 0.0]),
                (xyz[1], uv[1], [0.0, 1.0, 0.0]),
                (xyz[2], uv[2], [0.0, 0.0, 1.0]),
                (xyz[3], uv[3], [1.0, 0.0, 1.0])
            ].map(|(xyz, uv, rgb)| shader::Vertex { xyz: xyz, uv: uv, rgb: rgb });

            // Split the clockwise quad into two clockwise triangles.
            tri.push_all([v[0], v[1], v[2]]);
            tri.push_all([v[2], v[3], v[0]]);
        }
        renderer.create_buffer(tri.as_slice())
    };

    let mut events = GameIterator::new(&mut window, &game_iter_settings);
    for e in events {
        match e {
            Render(_args) => {
                renderer.set_view(first_person.camera(
                        _args.ext_dt * if extrapolate_time { 0.0 } else { 1.0 }
                    ).orthogonal());
                renderer.reset();
                renderer.render(buffer);
                renderer.end_frame();

                let fps = fps_counter.update();
                let title = format!("Hematite @ {}FPS", fps);
                events.game_window.window.set_title(title.as_slice());
            }
            Input(input::KeyPress { key: input::keyboard::C }) => {
                println!("Turned cursor capture {}", 
                    if capture_cursor { "off" } else { "on" });
                capture_cursor = !capture_cursor;

                events.game_window.capture_cursor(capture_cursor);
            },
            Input(input::KeyPress { key: input::keyboard::X }) => {
                println!("Turned extrapolated time {}", 
                    if extrapolate_time { "off" } else { "on" });
                extrapolate_time = !extrapolate_time;
            },
            Input(input::MouseRelativeMove { .. }) => {
                if !capture_cursor {
                    // Don't send the mouse event to the FPS controller.
                    continue;
                }
            }
            _ => {}
        }

        // Camera controller.
        match e {
            Input(ref args) => first_person.input(args),
            Update(piston::UpdateArgs { dt, .. }) => first_person.update(dt),
            _ => {}
        }
    }
}
