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
use hgl::{
    Shader,
    Program,
    Triangles,
    Vbo,
    Vao,
};

pub enum MinecraftTexture {
    Grass,
}

static VERTEX_SHADER: &'static str = r"
attribute vec3 position;
attribute vec4 fill_color;
attribute vec2 tex_coord;

uniform mat4 m_projection;
uniform mat4 m_view;
uniform mat4 m_model;
uniform sampler2D s_texture;

varying vec2 v_tex_coord;
varying vec4 v_fill_color;

void main() {
    v_tex_coord = tex_coord;
    v_fill_color = fill_color;
    gl_Position = m_projection * m_view * m_model * vec4(position, 1.0);
}
";

static FRAGMENT_SHADER: &'static str = r"
uniform sampler2D s_texture;

varying vec2 v_tex_coord;
varying vec4 v_fill_color;

void main() {
    gl_FragColor = texture2D(s_texture, v_tex_coord) * v_fill_color;
}
";

static TEST_TEXTURE: MinecraftTexture = Grass;

impl MinecraftTexture {
    pub fn src_xy(&self) -> (i32, i32) {
        match *self {
            Grass => (0, 0),
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
    
    // Load texture.
    let image = asset_store.path("minecraft-texture.png").unwrap();
    let image = Texture::from_path(&image).unwrap();
    let game_iter_settings = GameIteratorSettings {
            updates_per_second: 120,
            max_frames_per_second: 60,
        };
    let ref mut gl = Gl::new();

    // Compile shaders.
    let program = Program::link([
            Shader::compile(VERTEX_SHADER, hgl::VertexShader),
            Shader::compile(FRAGMENT_SHADER, hgl::FragmentShader)
        ]).unwrap();
    program.bind();

    for e in GameIterator::new(&mut window, &game_iter_settings) {
        match e {
            Render(args) => {
                gl.viewport(0, 0, args.width as i32, args.height as i32);

                let c = Context::abs(args.width as f64, args.height as f64);
                c.rgb(0.0, 0.0, 0.0).draw(gl);
                let (src_x, src_y) = TEST_TEXTURE.src_xy();
                c
                    .image(&image)
                    .src_rect(src_x * 16, src_y * 16, 16, 16)
                    .draw(gl);
            },
            _ => {}
        }
    }
}
