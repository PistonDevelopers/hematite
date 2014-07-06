#![feature(globs)]
#![feature(unsafe_destructor)]

extern crate debug;
extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate sdl2_game_window;
extern crate gl;
extern crate libc;
extern crate cgmath;

use Window = sdl2_game_window::GameWindowSDL2;
use graphics::*;
use piston::{
    AssetStore,
    GameIterator,
    GameIteratorSettings,
    GameWindowSettings,
    Render,
    Update,
};
use opengl_graphics::{
    Gl,
    Texture,
};

use cam::Camera;

pub mod shader;
pub mod cube;
pub mod quad;
pub mod cam;
pub mod texture;
pub mod vecmath;

static TEST_TEXTURE: texture::MinecraftTexture = texture::Grass;

fn main() {
    let mut window = Window::new(
        GameWindowSettings {
            title: "Hematite".to_string(),
            size: [300, 300], // [640, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );

    let asset_store = AssetStore::from_folder("assets");
    
    // Load texture.
    let texture = asset_store.path("minecraft-texture.png").unwrap();
    let ref texture = Texture::from_path(&texture).unwrap();
    let game_iter_settings = GameIteratorSettings {
            updates_per_second: 120,
            max_frames_per_second: 60,
        };
    let ref mut gl = Gl::new();

    let shader = shader::Shader::new();

    let mut camera = Camera {
            position: [0.0, 0.0, -0.5],
            target: [0.0, 0.0, 0.0],
            right: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0]
        };
    let mut t: f64 = 0.0;
    let radius = 0.5;
    let speed = 1.0;
 
    for e in GameIterator::new(&mut window, &game_iter_settings) {
        match e {
            Render(args) => {
                gl.viewport(0, 0, args.width as i32, args.height as i32);
                let c = Context::abs(args.width as f64, args.height as f64);
                c.rgb(0.0, 0.0, 0.0).draw(gl);

                // TEST
                // println!("Seconds: {}", t);
        
                camera.position = [
                        (speed * t).cos() * radius,
                        0.5,
                        (speed * t).sin() * radius                          
                    ];
                camera.update_right();

                // TEST
                // println!("Camera: {:?}", camera);

                let cam_mat = camera.orthogonal(); 
                let tex = TEST_TEXTURE;
                shader.render(gl, |ready_shader| {
                    let quad = tex.to_quad(texture, cam_mat);

                    // TEST
                    // println!("{:?}", quad);

                    quad.render(ready_shader);
                });
                let (src_x, src_y) = tex.get_src_xy();
                c.image(texture)
                .src_rect(src_x * 16, src_y * 16, 16, 16)
                .draw(gl);
            },
            Update(args) => {
                t += args.dt;
            },
            _ => {}
        }
    }
}
