use gl;
use gl::types::GLint;
use hgl;
use hgl::{Program, Triangles, Vbo, Vao};
use vecmath::Matrix4;

use std::cell::Cell;
use std::mem;

macro_rules! make_vertex_shader {
    ($version:expr $($profile:ident)*) => (concat!("
        #version ", stringify!($version), $(stringify!($profile),)* "

        #if __VERSION__ < 130
            #define in attribute
            #define out varying
        #endif

        uniform mat4 projection, view;

        in vec2 tex_coord;
        in vec3 color, position;

        out vec2 v_tex_coord;
        out vec3 v_color;

        void main() {
            v_tex_coord = tex_coord;
            v_color = color;
            gl_Position = projection * view * vec4(position, 1.0);
        }
    "))
}

macro_rules! make_fragment_shader {
    ($version:expr $($profile:ident)*) => (concat!("
        #version ", stringify!($version), $(stringify!($profile),)* "

        #if __VERSION__ < 130
            #define in varying
            #define texture texture2D
            #define out_color gl_FragColor
        #else
            out vec4 out_color;
        #endif

        uniform sampler2D s_texture;

        in vec2 v_tex_coord;
        in vec3 v_color;

        void main() {
            vec4 tex_color = texture(s_texture, v_tex_coord);
            if(tex_color.a == 0.0) // Discard transparent pixels.
                discard;
            out_color = tex_color * vec4(v_color, 1.0);
        }
    "))
}

macro_rules! make_shaders {
    ($version:expr $($profile:ident)*) => (
        (make_vertex_shader!($version $($profile)*), make_fragment_shader!($version $($profile)*))
    )
}

pub struct Shader {
    program: Program,
    vao: Vao,
    projection_uniform: GLint,
    view_uniform: GLint
}

pub struct Vertex {
    pub xyz: [f32, ..3],
    pub uv: [f32, ..2],
    pub rgb: [f32, ..3]
}

pub struct Buffer {
    vbo: Vbo,
    triangles: Cell<uint>
}

impl Shader {
    pub fn new() -> Shader {
        let vao = Vao::new();
        vao.bind();

        let (vertex_shader, fragment_shader) = make_shaders!(330 core);

        let program = Program::link([
            hgl::Shader::compile(vertex_shader, hgl::VertexShader),
            hgl::Shader::compile(fragment_shader, hgl::FragmentShader)
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
        gl::DepthFunc(gl::LEQUAL);
        gl::Enable(gl::CULL_FACE);
        gl::CullFace(gl::FRONT);
        gl::ClearColor(0.81, 0.8, 1.0, 1.0);
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

    pub fn render(&self, buffer: &Buffer) {
        buffer.vbo.bind();
        let s_f32 = mem::size_of::<f32>();
        let total = 8*s_f32 as i32;
        self.vao.enable_attrib(&self.program, "position", gl::FLOAT, 3, total, 0);
        self.vao.enable_attrib(&self.program, "tex_coord", gl::FLOAT, 2, total, 3*s_f32);
        self.vao.enable_attrib(&self.program, "color", gl::FLOAT, 3, total, 5*s_f32);
        self.vao.draw_array(Triangles, 0, (buffer.triangles.get() * 3) as gl::types::GLint);
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            vbo: Vbo::new(),
            triangles: Cell::new(0)
        }
    }

    pub fn load_data(&self, data: &[[Vertex, ..3]]) {
        self.vbo.load_data(data, hgl::buffer::DynamicDraw);
        self.triangles.set(data.len());
    }
}
