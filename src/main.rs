#![feature(globs, macro_rules, phase)]

extern crate debug;
extern crate piston;
extern crate sdl2;
extern crate sdl2_game_window;
extern crate gfx;
extern crate device;
#[phase(plugin)]
extern crate gfx_macros;
extern crate image;
extern crate libc;
extern crate cgmath;
extern crate time;

extern crate flate;
extern crate native;
extern crate rustrt;
extern crate serialize;

use sdl2_game_window::GameWindowSDL2 as Window;
use piston::input;
use piston::cam;
use piston::vecmath::{vec3_add, vec3_scale, vec3_normalized};
use piston::{
    AssetStore,
    GameIterator,
    GameIteratorSettings,
    GameWindow,
    GameWindowSettings,
    Input,
    Render,
    Update
};

use array::*;

use std::cmp::max;
use std::f32::INFINITY;
use std::f32::consts::PI;
use std::io::fs::File;

pub mod array;
pub mod chunk;
pub mod cube;
pub mod fps_counter;
pub mod shader;
pub mod texture;

pub mod minecraft {
    pub use self::data_1_8_pre2 as data;

    mod data_1_8_pre2;
    pub mod biome;
    pub mod block_state;
    pub mod model;
    pub mod nbt;
    pub mod region;
}

fn main() {
    let world = Path::new(std::os::args().as_slice().get(1).expect("Usage: ./hematite <path/to/world>").as_slice());

    let level = minecraft::nbt::Nbt::from_gzip(File::open(&world.join("level.dat")).read_to_end().unwrap().as_slice()).unwrap();
    println!("{}", level);
    let player_pos: [f32, ..3] = Array::from_iter(level["Data"]["Player"]["Pos"].as_double_list().unwrap().iter().map(|&x| x as f32));
    let player_chunk = [player_pos.x(), player_pos.z()].map(|x| (x / 16.0).floor() as i32);
    let player_rot = level["Data"]["Player"]["Rotation"].as_float_list().unwrap();
    let player_yaw = player_rot[0];
    let player_pitch = player_rot[1];

    let [region_x, region_z] = player_chunk.map(|x| x >> 5);
    let region_file = world.join(format!("region/r.{}.{}.mca", region_x, region_z));
    let region = minecraft::region::Region::open(&region_file);

    let mut window = Window::new(
        piston::shader_version::opengl::OpenGL_3_3,
        GameWindowSettings {
            title: format!("Hematite - {}", world.display()),
            size: [854, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );
    let (mut device, frame) = window.gfx();

    let assets = &AssetStore::from_folder("../assets");

    // Load biomes.
    let biomes = minecraft::biome::Biomes::load(assets);

    // Load block state definitions and models.
    let block_states = minecraft::block_state::BlockStates::load(assets, &mut device);

    let mut renderer = shader::Renderer::new(device, frame, block_states.texture().tex);

    let mut chunk_manager = chunk::ChunkManager::new();

    println!("Started loading chunks...");
    let [cx_base, cz_base] = player_chunk.map(|x| max(0, (x & 0x1f) - 8) as u8);
    for cz in range(cz_base, cz_base + 16) {
        for cx in range(cx_base, cx_base + 16) {
            match region.get_chunk_column(cx, cz) {
                Some(column) => {
                    let [cx, cz] = [cx as i32 + region_x * 32, cz as i32 + region_z * 32];
                    chunk_manager.add_chunk_column(cx, cz, column)
                }
                None => {}
            }
        }
    }
    println!("Finished loading chunks.");

    let projection_mat = cam::CameraPerspective {
        fov: 70.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: {
            let (w, h) = window.get_size();
            (w as f32) / (h as f32)
        }
    }.projection();
    renderer.set_projection(projection_mat);

    let mut first_person_settings = cam::FirstPersonSettings::keyboard_wasd();
    first_person_settings.speed_horizontal = 8.0;
    first_person_settings.speed_vertical = 4.0;
    let mut first_person = cam::FirstPerson::new(
        player_pos,
        first_person_settings
    );
    first_person.yaw = PI - player_yaw / 180.0 * PI;
    first_person.pitch = player_pitch / 180.0 * PI;

    let game_iter_settings = GameIteratorSettings {
        updates_per_second: 120,
        max_frames_per_second: 10000
    };

    // Disable V-Sync.
    sdl2::video::gl_set_swap_interval(0);

    let mut fps_counter = fps_counter::FPSCounter::new();

    let mut pending_chunks = vec![];
    chunk_manager.each_chunk_and_neighbors(|coords, buffer, chunks, column_biomes| {
        pending_chunks.push((coords, buffer, chunks, column_biomes));
    });

    let mut capture_cursor = false;
    println!("Press C to capture mouse");

    let mut staging_buffer = vec![];
    let mut last_render = time::precise_time_ns();
    let mut events = GameIterator::new(&mut window, &game_iter_settings);
    for e in events {
        match e {
            Render(_) => {
                // Update camera.
                let now = time::precise_time_ns();
                let dt = (now - last_render) as f64 / 1_000_000_000.0f64;
                first_person.update(dt);
                last_render = now;

                // Apply the same y/z camera offset vanilla minecraft has.
                let mut camera = first_person.camera(0.0);
                camera.position[1] += 1.62;
                let mut xz_forward = camera.forward;
                xz_forward[1] = 0.0;
                xz_forward = vec3_normalized(xz_forward);
                camera.position = vec3_add(camera.position, vec3_scale(xz_forward, 0.1));

                let view_mat = camera.orthogonal();
                renderer.set_view(view_mat);
                renderer.clear();
                let mut num_chunks = 0u;
                let mut num_total_chunks = 0u;
                chunk_manager.each_chunk(|cx, cy, cz, _, buffer| {
                    match buffer {
                        Some(buffer) => {
                            num_total_chunks += 1;

                            let inf = INFINITY;
                            let mut bb_min = [inf, inf, inf];
                            let mut bb_max = [-inf, -inf, -inf];
                            let xyz = [cx, cy, cz].map(|x| x as f32 * 16.0);
                            for &dx in [0.0, 16.0].iter() {
                                for &dy in [0.0, 16.0].iter() {
                                    for &dz in [0.0, 16.0].iter() {
                                        use piston::vecmath::col_mat4_transform;
                                        let [x, y, z] = vec3_add(xyz, [dx, dy, dz]);
                                        let xyzw = col_mat4_transform(view_mat, [x, y, z, 1.0]);
                                        let [x, y, z, w] = col_mat4_transform(projection_mat, xyzw);
                                        let xyz = vec3_scale([x, y, z], 1.0 / w);
                                        bb_min = Array::from_fn(|i| bb_min[i].min(xyz[i]));
                                        bb_max = Array::from_fn(|i| bb_max[i].max(xyz[i]));
                                    }
                                }
                            }

                            let cull_bits: [bool, ..3] = Array::from_fn(|i| {
                                let (min, max) = (bb_min[i], bb_max[i]);
                                min.signum() == max.signum() && min.abs().min(max.abs()) >= 1.0
                            });

                            if !cull_bits.iter().any(|&cull| cull) {
                                renderer.render(buffer);
                                num_chunks += 1;
                            }
                        }
                        None => {}
                    }
                });
                renderer.end_frame();

                let fps = fps_counter.update();
                let title = format!("Hematite w/ {}/{}C @ {}FPS - {}",
                                    num_chunks, num_total_chunks, fps, world.display());
                events.game_window.window.set_title(title.as_slice());
            }
            Update(_) => {
                // HACK(eddyb) find the closest chunk to the player.
                // The pending vector should be sorted instead.
                let closest = pending_chunks.iter().enumerate().min_by(|&(_, &([cx, cy, cz], _, _, _))| {
                    let [px, py, pz] = first_person.position.map(|x| (x / 16.0).floor() as i32);
                    let [x2, y2, z2] = [cx - px, cy - py, cz - pz].map(|x| x * x);
                    x2 + y2 + z2
                }).map(|(i, _)| i);

                let pending = closest.and_then(|i| pending_chunks.swap_remove(i));
                match pending {
                    Some((coords, buffer, chunks, column_biomes)) => {
                        match buffer.get() {
                            Some(buffer) => renderer.delete_buffer(buffer),
                            None => {}
                        }
                        minecraft::block_state::fill_buffer(&block_states, &biomes,
                                                            &mut staging_buffer,
                                                            coords, chunks,
                                                            column_biomes);
                        buffer.set(Some(renderer.create_buffer(staging_buffer.as_slice())));
                        staging_buffer.clear();

                        if pending_chunks.is_empty() {
                            println!("Finished filling chunk vertex buffers.");
                        }
                    }
                    None => {}
                }
            }
            Input(input::Press(input::Keyboard(input::keyboard::C))) => {
                println!("Turned cursor capture {}", 
                    if capture_cursor { "off" } else { "on" });
                capture_cursor = !capture_cursor;

                events.game_window.capture_cursor(capture_cursor);
            }
            Input(input::Move(input::MouseRelative(_, _))) => {
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
            _ => {}
        }
    }
}
