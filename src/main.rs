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
attribute vec3 fill_color;
attribute vec2 tex_coord;

uniform mat4 m_projection;
uniform mat4 m_view;
uniform mat4 m_model;
uniform sampler2D s_texture;

varying vec2 v_tex_coord;
varying vec4 v_fill_color;

void main() {
    v_tex_coord = tex_coord;
    v_fill_color = vec4(fill_color, 1.0);
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

    /*
            2  --------- 3
              /       / |
             /       /  |
         7  -------- 6  | 1
           |        |  /
           |        | /
           |        |/
         4  -------- 5

          
           ---- ---- ---- ----
          |    |    |    |    |
          |    |    |    |    |
           ---- ---- ---- ----
    */

    let cube_quads = vec![
        4u, 5, 6, 7,
        5, 1, 3, 6,
        1, 0, 2, 3,
        0, 4, 7, 2,
        7, 6, 3, 2,
        0, 1, 5, 4,
    ];

    // Cube vertices.
    let cube_vertices = [
        // This is the back surface
        -1.0f32,    -1.0,       1.0, // 0
         1.0,       -1.0,       1.0, // 1
         1.0,        1.0,       1.0, // 2
        -1.0,        1.0,       1.0, // 3

        // This is the front surface
        -1.0,       -1.0,      -1.0, // 4
         1.0,       -1.0,      -1.0, // 5
         1.0,        1.0,      -1.0, // 6
        -1.0,        1.0,      -1.0  // 7
    ];

    /*
    // Initialize a vertex array buffer with the cube vertices.
    let vao = Vao::new();
    vao.bind();
    vao.enable_attrib(&program, "position", gl::FLOAT, 3, 3 * size_of::<f32>() as i32, 0);
    cube_vbo.bind();    

    // vao.enable_attrib(&program, "fill_color", gl::FLOAT, 3, 3 * size_of::<f32>() as i32, 0);
    // vao.enable_attrib(&program, "tex_coord", gl::FLOAT, 2, 2 * size_of::<f32>() as i32, 0);
    */
    
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
