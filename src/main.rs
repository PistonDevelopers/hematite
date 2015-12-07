extern crate byteorder;
extern crate camera_controllers;
extern crate docopt;
extern crate piston;
extern crate flate2;
extern crate fps_counter;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_voxel;
extern crate image;
extern crate libc;
extern crate memmap;
extern crate rustc_serialize;
extern crate sdl2;
extern crate sdl2_window;
extern crate shader_version;
extern crate time;
extern crate vecmath;
extern crate zip;

use gfx::traits::{Device, Stream, StreamFactory};
// Reexport modules from gfx_voxel while stuff is moving
// from Hematite to the library.
pub use gfx_voxel::{ array, cube };

use std::cell::RefCell;
use std::cmp::max;
use std::f32::consts::PI;
use std::f32::INFINITY;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use array::*;
use docopt::Docopt;
use piston::event_loop::{ Events, EventLoop };
use flate2::read::GzDecoder;
use sdl2_window::Sdl2Window;
use shader::Renderer;
use vecmath::{ vec3_add, vec3_scale};
use piston::window::{ Size, Window, AdvancedWindow, OpenGLWindow,
    WindowSettings };

pub mod minecraft;
pub mod chunk;
pub mod shader;

use minecraft::*;
use minecraft::biome::Biomes;
use minecraft::block_state::BlockStates;

static USAGE: &'static str = "
hematite, Minecraft made in Rust!

Usage:
    hematite [options] <world>

Options:
    -p, --path               Fully qualified path for world folder.
    --mcversion=<version>    Minecraft version [default: 1.8.3].
";

#[derive(RustcDecodable)]
struct Args {
    arg_world: String,
    flag_path: bool,
    flag_mcversion: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|dopt| dopt.decode())
                            .unwrap_or_else(|e| e.exit());

    // Automagically pull MC assets
    minecraft::fetch_assets(&args.flag_mcversion);

    // Automagically expand path if world is located at
    // $MINECRAFT_ROOT/saves/<world_name>
    let world = if args.flag_path {
        PathBuf::from(&args.arg_world)
    } else {
        let mut mc_path = minecraft::vanilla_root_path();
        mc_path.push("saves");
        mc_path.push(args.arg_world);
        mc_path
    };

    let file_name = PathBuf::from(world.join("level.dat"));
    let level_reader = GzDecoder::new(File::open(file_name).unwrap()).unwrap();
    let level = minecraft::nbt::Nbt::from_reader(level_reader).unwrap();
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

    let regions = player_chunk.map(|x| x >> 5);
    let region_file = world.join(
            format!("region/r.{}.{}.mca", regions[0], regions[1])
        );
    let region = minecraft::region::Region::open(&region_file).unwrap();

    let loading_title = format!(
            "Hematite loading... - {}",
            world.file_name().unwrap().to_str().unwrap()
        );
    let mut window: Sdl2Window = WindowSettings::new(
            loading_title,
            Size { width: 854, height: 480 })
            .fullscreen(false)
            .exit_on_esc(true)
            .samples(0)
            .vsync(false)
            .build()
            .unwrap();

    let (mut device, mut factory) = gfx_device_gl::create(|s|
        window.get_proc_address(s) as *const _
    );

    let Size { width: w, height: h } = window.size();
    let frame = factory.make_fake_output(w as u16, h as u16);
    let stream: gfx::OwnedStream<gfx_device_gl::Device, _> = factory.create_stream(frame);

    let assets = Path::new("./assets");

    // Load biomes.
    let biomes = Biomes::load(&assets);

    // Load block state definitions and models.
    let block_states = BlockStates::load(&assets, &mut factory);

    let mut renderer = Renderer::new(factory, stream, block_states.texture.handle());

    let mut chunk_manager = chunk::ChunkManager::new();

    println!("Started loading chunks...");
    let c_bases = player_chunk.map(|x| max(0, (x & 0x1f) - 8) as u8);
    for cz in c_bases[1]..c_bases[1] + 16 {
        for cx in c_bases[0]..c_bases[0] + 16 {
            match region.get_chunk_column(cx, cz) {
                Some(column) => {
                    let (cx, cz) = (
                        cx as i32 + regions[0] * 32,
                        cz as i32 + regions[1] * 32
                    );
                    chunk_manager.add_chunk_column(cx, cz, column)
                }
                None => {}
            }
        }
    }
    println!("Finished loading chunks.");

    let projection_mat = camera_controllers::CameraPerspective {
        fov: 70.0,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: {
            let Size { width: w, height: h } = window.size();
            (w as f32) / (h as f32)
        }
    }.projection();
    renderer.set_projection(projection_mat);

    let mut first_person_settings = camera_controllers::FirstPersonSettings::keyboard_wasd();
    first_person_settings.speed_horizontal = 8.0;
    first_person_settings.speed_vertical = 4.0;
    let mut first_person = camera_controllers::FirstPerson::new(
        player_pos,
        first_person_settings
    );
    first_person.yaw = PI - player_yaw / 180.0 * PI;
    first_person.pitch = player_pitch / 180.0 * PI;

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
    let ref window = Rc::new(RefCell::new(window));
    for e in window.clone().events()
        .ups(120)
        .max_fps(10_000)
    {
        use piston::input::Button::Keyboard;
        use piston::input::Input::{ Move, Press };
        use piston::input::keyboard::Key;
        use piston::input::Motion::MouseRelative;
        use piston::input::Event;

        match e {
            Event::Render(_) => {
                // Apply the same y/z camera offset vanilla minecraft has.
                let mut camera = first_person.camera(0.0);
                camera.position[1] += 1.62;
                let view_mat = camera.orthogonal();
                renderer.set_view(view_mat);
                renderer.clear();
                let mut num_chunks: usize = 0;
                let mut num_sorted_chunks: usize = 0;
                let mut num_total_chunks: usize = 0;
                let start_time = time::precise_time_ns();
                chunk_manager.each_chunk(|cx, cy, cz, _, buffer| {
                    match buffer.borrow_mut().as_mut() {
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

                                        let v = vec3_add(xyz, [dx, dy, dz]);
                                        let xyzw = col_mat4_transform(view_mat, [v[0], v[1], v[2], 1.0]);
                                        let v = col_mat4_transform(projection_mat, xyzw);
                                        let xyz = vec3_scale([v[0], v[1], v[2]], 1.0 / v[3]);
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
                renderer.stream.flush(&mut device);
                let frame_end_time = time::precise_time_ns();

                let fps = fps_counter.tick();
                let title = format!(
"Hematite sort={} render={} total={} in {:.2}ms+{:.2}ms @ {}FPS - {}",
                        num_sorted_chunks,
                        num_chunks,
                        num_total_chunks,
                        (end_time - start_time) as f64 / 1e6,
                        (frame_end_time - end_time) as f64 / 1e6,
                        fps, world.file_name().unwrap().to_str().unwrap()
                    );
                window.borrow_mut().set_title(title);
            }
            Event::AfterRender(_) => {
                device.cleanup();
            }
            Event::Update(_) => {
                use std::i32;
                // HACK(eddyb) find the closest chunk to the player.
                // The pending vector should be sorted instead.
                let pp = first_person.position.map(|x| (x / 16.0).floor() as i32);
                let closest = pending_chunks.iter().enumerate().fold(
                    (None, i32::max_value()),
                    |(best_i, best_dist), (i, &(cc, _, _, _))| {
                        let xyz = [cc[0] - pp[0], cc[1] - pp[1], cc[2] - pp[2]]
                            .map(|x| x * x);
                        let dist = xyz[0] + xyz[1] + xyz[2];
                        if dist < best_dist {
                            (Some(i), dist)
                        } else {
                            (best_i, best_dist)
                        }
                    }
                ).0;
                let pending = closest.and_then(|i| {
                    // Vec swap_remove doesn't return Option anymore
                    match pending_chunks.len() {
                        0 => None,
                        _ => Some(pending_chunks.swap_remove(i))
                    }
                });
                match pending {
                    Some((coords, buffer, chunks, column_biomes)) => {
                        minecraft::block_state::fill_buffer(
                            &block_states, &biomes, &mut staging_buffer,
                            coords, chunks, column_biomes
                        );
                        *buffer.borrow_mut() = Some(
                            renderer.create_buffer(&staging_buffer[..])
                        );
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

                window.borrow_mut().set_capture_cursor(capture_cursor);
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
