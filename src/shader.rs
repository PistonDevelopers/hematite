use gl;
use gl::types::GLint;
use hgl;
use hgl::{Program, Triangles, Vbo, Vao};
use vecmath::Matrix4;

use std::mem;

static VERTEX_SHADER: &'static str = r"
    uniform mat4 projection, view;

    attribute vec2 tex_coord;
    attribute vec3 color;
    attribute vec3 position;

    varying vec2 v_tex_coord;
    varying vec3 v_color;

    void main() {
        v_tex_coord = tex_coord;
        v_color = color;
        gl_Position = projection * view * vec4(position, 1.0);
    }
";

static FRAGMENT_SHADER: &'static str = r"
    uniform sampler2D s_texture;

    varying vec2 v_tex_coord;
    varying vec3 v_color;

    void main() {
        gl_FragColor = texture2D(s_texture, v_tex_coord) * vec4(v_color, 1.0);
    }
";

pub struct Shader {
    program: Program,
    vao: Vao,
    projection_uniform: GLint,
    view_uniform: GLint
}

pub struct Buffer {
    vbo: Vbo,
    triangles: uint
}

impl Shader {
    pub fn new() -> Shader {
        let vao = Vao::new();
        vao.bind();

        let program = Program::link([
            hgl::Shader::compile(VERTEX_SHADER, hgl::VertexShader),
            hgl::Shader::compile(FRAGMENT_SHADER, hgl::FragmentShader)
        ]).unwrap();
        program.bind();

        let projection_uniform = program.uniform("projection");
        let view_uniform = program.uniform("view");

        Shader {
            program: program,
            vao: vao,
            projection_uniform: projection_uniform,
            view_uniform: view_uniform
        }
    }

    pub fn bind(&self) {
        self.vao.bind();
        self.program.bind();
        gl::Enable(gl::DEPTH_TEST);
    }

    pub fn set_projection(&self, proj_mat: Matrix4) {
        unsafe {
            gl::UniformMatrix4fv(self.projection_uniform, 1, gl::FALSE, proj_mat[0].as_ptr());
        }
    }

    pub fn set_view(&self, view_mat: Matrix4) {
        unsafe {
            gl::UniformMatrix4fv(self.view_uniform, 1, gl::FALSE, view_mat[0].as_ptr());
        }
    }

    pub fn new_buffer(&self) -> Buffer {
        let vbo = Vbo::new();
        vbo.bind();
        let s_f32 = mem::size_of::<f32>();
        self.vao.enable_attrib(&self.program, "position", gl::FLOAT, 3, 8*s_f32 as i32, 0);
        self.vao.enable_attrib(&self.program, "tex_coord", gl::FLOAT, 2, 8*s_f32 as i32, 3*s_f32);
        self.vao.enable_attrib(&self.program, "color", gl::FLOAT, 3, 8*s_f32 as i32, 5*s_f32);
        Buffer {
            vbo: vbo,
            triangles: 0
        }
    }

    pub fn render(&self, buffer: &Buffer) {
        buffer.vbo.bind();
        self.vao.draw_array(Triangles, 0, (buffer.triangles * 3) as gl::types::GLint);
    }
}

impl Buffer {
    pub fn load_data(&mut self, data: &[[([f32, ..3], [f32, ..2], [f32, ..3]), ..3]]) {
        self.vbo.load_data(data, hgl::buffer::DynamicDraw);
        self.triangles = data.len();
    }
}
