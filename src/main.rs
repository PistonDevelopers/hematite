#![feature(box_syntax)]
#![feature(plugin)]

#![feature(collections, core, io, os, path, rustc_private, std_misc)]

#![plugin(gfx_macros)]
#[no_link]

#[macro_use]
extern crate gfx_macros;
extern crate cam;
extern crate current;
extern crate event;
extern crate flate;
extern crate fps_counter;
extern crate gfx;
extern crate "gfx_device_gl" as device;
extern crate gfx_macros;
extern crate gfx_voxel;
extern crate image;
extern crate input;
extern crate quack;
extern crate sdl2;
extern crate sdl2_window;
extern crate shader_version;
extern crate time;
extern crate vecmath;
extern crate window;

extern crate "rustc-serialize" as serialize;

// Reexport modules from gfx_voxel while stuff is moving
// from Hematite to the library.
pub use gfx_voxel::{ array, cube, texture };

use std::cell::RefCell;
use std::cmp::max;
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::old_io::fs::File;
use std::num::Float;

use array::*;
use event::{ Event, Events, MaxFps, Ups };
use quack::{Get, Set};
use sdl2_window::Sdl2Window;
use shader::Renderer;
use vecmath::{ vec3_add, vec3_scale, vec3_normalized };
use window::{ CaptureCursor, Size, WindowSettings };

use minecraft::biome::Biomes;
use minecraft::block_state::BlockStates;

pub mod chunk;
pub mod shader;

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
    let args = std::os::args();
    let world = args.as_slice().get(1).expect(
            "Usage: ./hematite <path/to/world>"
        ).as_slice();
    let world = Path::new(world);

    let level_gzip = File::open(&world.join("level.dat"))
        .read_to_end().unwrap();
    let level = minecraft::nbt::Nbt::from_gzip(level_gzip.as_slice())
        .unwrap();
    println!("{:?}", level);
    let player_pos: [f32; 3] = Array::from_iter(
            level["Data"]["Player"]["Pos"]
            .as_double_list().unwrap().iter().map(|&x| x as f32)
        );
    let player_chunk = [player_pos.x(), player_pos.z()]
        .map(|x| (x / 16.0).floor() as i32);
    let player_rot = level["Data"]["Player"]["Rotation"]
        .as_float_list().unwrap();
    let player_yaw = player_rot[0];
    let player_pitch = player_rot[1];

    let [region_x, region_z] = player_chunk.map(|x| x >> 5);
    let region_file = world.join(
            format!("region/r.{}.{}.mca", region_x, region_z)
        );
    let region = minecraft::region::Region::open(&region_file).unwrap();

    let loading_title = format!(
            "Hematite loading... - {}",
            world.filename_display()
        );
    let window = Sdl2Window::new(
        shader_version::OpenGL::_3_3,
        WindowSettings {
            title: loading_title,
            size: [854, 480],
            fullscreen: false,
            exit_on_esc: true,
            samples: 0,
        }
    );
    let mut device = gfx::GlDevice::new(|s| unsafe {
        std::mem::transmute(sdl2::video::gl_get_proc_address(s))
    });
    let Size([w, h]) = window.get();
    let frame = gfx::Frame::new(w as u16, h as u16);

    let assets = Path::new("./assets");

    // Load biomes.
    let biomes = Biomes::load(&assets);

    // Load block state definitions and models.
    let block_states = BlockStates::load(&assets, &mut device);

    let mut renderer = Renderer::new(device, frame, block_states.texture().handle);

    let mut chunk_manager = chunk::ChunkManager::new();

    println!("Started loading chunks...");
    let [cx_base, cz_base] = player_chunk.map(|x| max(0, (x & 0x1f) - 8) as u8);
    for cz in range(cz_base, cz_base + 16) {
        for cx in range(cx_base, cx_base + 16) {
            match region.get_chunk_column(cx, cz) {
                Some(column) => {
                    let [cx, cz] = [
                        cx as i32 + region_x * 32,
                        cz as i32 + region_z * 32
                    ];
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
            let Size([w, h]) = window.get();
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

    // Disable V-Sync.
    sdl2::video::gl_set_swap_interval(0);

    let mut fps_counter = fps_counter::FPSCounter::new();

    let mut pending_chunks = vec![];
    chunk_manager.each_chunk_and_neighbors(
        |coords, buffer, chunks, column_biomes| {
            pending_chunks.push((coords, buffer, chunks, column_biomes));
        }
    );

    let mut capture_cursor = false;
    println!("Press C to capture mouse");

    let mut staging_buffer = vec![];
    let ref window = RefCell::new(window);
    for e in Events::new(window)
        .set(Ups(120))
        .set(MaxFps(10_000)) {
        use input::Button::Keyboard;
        use input::Input::{ Move, Press };
        use input::keyboard::Key;
        use input::Motion::MouseRelative;

        match e {
            Event::Render(_) => {
                // Apply the same y/z camera offset vanilla minecraft has.
                let mut camera = first_person.camera(0.0);
                camera.position[1] += 1.62;
                let mut xz_forward = camera.forward;
                xz_forward[1] = 0.0;
                xz_forward = vec3_normalized(xz_forward);
                camera.position = vec3_add(
                    camera.position,
                    vec3_scale(xz_forward, 0.1)
                );

                let view_mat = camera.orthogonal();
                renderer.set_view(view_mat);
                renderer.clear();
                let mut num_chunks = 0us;
                let mut num_sorted_chunks = 0us;
                let mut num_total_chunks = 0us;
                let start_time = time::precise_time_ns();
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
                                        use vecmath::col_mat4_transform;

                                        let [x, y, z] = vec3_add(xyz, [dx, dy, dz]);
                                        let xyzw = col_mat4_transform(view_mat, [x, y, z, 1.0]);
                                        let [x, y, z, w] = col_mat4_transform(projection_mat, xyzw);
                                        let xyz = vec3_scale([x, y, z], 1.0 / w);
                                        bb_min = Array::from_fn(|i| bb_min[i].min(xyz[i]));
                                        bb_max = Array::from_fn(|i| bb_max[i].max(xyz[i]));
                                    }
                                }
                            }

                            let cull_bits: [bool; 3] = Array::from_fn(|i| {
                                let (min, max) = (bb_min[i], bb_max[i]);
                                min.signum() == max.signum()
                                    && min.abs().min(max.abs()) >= 1.0
                            });

                            if !cull_bits.iter().any(|&cull| cull) {
                                renderer.render(buffer);
                                num_chunks += 1;

                                if bb_min[0] < 0.0 && bb_max[0] > 0.0
                                || bb_min[1] < 0.0 && bb_max[1] > 0.0 {
                                    num_sorted_chunks += 1;
                                }
                            }
                        }
                        None => {}
                    }
                });
                let end_time = time::precise_time_ns();
                renderer.end_frame();
                let frame_end_time = time::precise_time_ns();

                let fps = fps_counter.tick();
                let title = format!(
"Hematite sort={} render={} total={} in {:.2}ms+{:.2}ms @ {}FPS - {}",
                        num_sorted_chunks,
                        num_chunks,
                        num_total_chunks,
                        (end_time - start_time) as f64 / 1e6,
                        (frame_end_time - end_time) as f64 / 1e6,
                        fps, world.filename_display()
                    );
                window.borrow_mut().window.set_title(title.as_slice());
            }
            Event::Update(_) => {
                // HACK(eddyb) find the closest chunk to the player.
                // The pending vector should be sorted instead.
                let closest = pending_chunks.iter().enumerate().min_by(
                    |&(_, &([cx, cy, cz], _, _, _))| {
                        let [px, py, pz] = first_person.position.map(|x|
                            (x / 16.0).floor() as i32);
                        let [x2, y2, z2] = [cx - px, cy - py, cz - pz]
                            .map(|x| x * x);
                        x2 + y2 + z2
                    }
                ).map(|(i, _)| i);

                let pending = closest.and_then(|i| {
                    // Vec swap_remove doesn't return Option anymore
                    match pending_chunks.len() {
                        0 => None,
                        _ => Some(pending_chunks.swap_remove(i))
                    }
                });
                match pending {
                    Some((coords, buffer, chunks, column_biomes)) => {
                        match buffer.get() {
                            Some(buffer) => renderer.delete_buffer(buffer),
                            None => {}
                        }
                        minecraft::block_state::fill_buffer(
                            &block_states, &biomes, &mut staging_buffer,
                            coords, chunks, column_biomes
                        );
                        buffer.set(Some(
                            renderer.create_buffer(staging_buffer.as_slice())
                        ));
                        staging_buffer.clear();

                        if pending_chunks.is_empty() {
                            println!("Finished filling chunk vertex buffers.");
                        }
                    }
                    None => {}
                }
            }
            Event::Input(Press(Keyboard(Key::C))) => {
                println!("Turned cursor capture {}",
                    if capture_cursor { "off" } else { "on" });
                capture_cursor = !capture_cursor;

                window.set(CaptureCursor(capture_cursor));
            }
            Event::Input(Move(MouseRelative(_, _))) => {
                if !capture_cursor {
                    // Don't send the mouse event to the FPS controller.
                    continue;
                }
            }
            _ => {}
        }

        first_person.event(&e);
    }
}
