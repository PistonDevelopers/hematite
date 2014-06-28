#![feature(globs)]

extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate sdl2_game_window;
extern crate hgl;

use Window = sdl2_game_window::GameWindowSDL2;
use graphics::*;
use piston::{
    AssetStore,
    GameIterator,
    GameIteratorSettings,
    GameWindowSettings,
    Render,
};
use opengl_graphics::{
    Gl,
    Texture,
};

pub enum MinecraftTexture {
    Lava,
}

impl MinecraftTexture {
    pub fn src_xy(&self) -> (i32, i32) {
        match *self {
            Lava => (0, 0)
        }
    }
}

fn main() {
    let mut window = Window::new(
        GameWindowSettings {
            title: "Hematite".to_string(),
            size: [640, 480],
            fullscreen: false,
            exit_on_esc: true,
        }
    );

    let asset_store = AssetStore::from_folder("assets");
    let image = asset_store.path("minecraft-texture.png").unwrap();
    let image = Texture::from_path(&image).unwrap();
    let game_iter_settings = GameIteratorSettings {
            updates_per_second: 120,
            max_frames_per_second: 60,
        };
    let ref mut gl = Gl::new();
    for e in GameIterator::new(&mut window, &game_iter_settings) {
        match e {
            Render(args) => {
                gl.viewport(0, 0, args.width as i32, args.height as i32);

                let c = Context::abs(args.width as f64, args.height as f64);
                c.rgb(0.0, 0.0, 0.0).draw(gl);
                let (src_x, src_y) = Lava.src_xy();
                c
                    .image(&image)
                    .src_rect(src_x * 16, src_y * 16, 16, 16)
                    .draw(gl);
            },
            _ => {}
        }
    }
}
