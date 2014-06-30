
use opengl_graphics::shader_utils::{
    DynamicAttribute,
    compile_shader,
};
use opengl_graphics::{
    Gl,
};
use gl;
use gl::types::{
    GLboolean,
    GLenum,
    GLuint,
    GLsizei,
    GLsizeiptr,
};
use std::mem;
use std::ptr;

pub enum NotReady {}
pub enum Ready {}

static VERTEX_SHADER: &'static str = r"
#version 330
in vec3 position;
in vec3 fill_color;
in vec2 tex_coord;

// TEST
// uniform mat4 m_projection;
// uniform mat4 m_view;
// uniform mat4 m_model;
uniform sampler2D s_texture;

out vec2 v_tex_coord;
out vec4 v_fill_color;

void main() {
    v_tex_coord = tex_coord;
    v_fill_color = vec4(fill_color, 1.0);
    // TEST
    gl_Position = vec4(position, 1.0); // m_projection * m_view * m_model * vec4(position, 1.0);
}
";

static FRAGMENT_SHADER: &'static str = r"
#version 330
out vec4 out_color;
uniform sampler2D s_texture;

in vec2 v_tex_coord;
in vec4 v_fill_color;

void main() {
    out_color = texture(s_texture, v_tex_coord) * v_fill_color;
}
";

pub struct Shader<State> {
    vao: GLuint,
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,
    position: DynamicAttribute,
    fill_color: DynamicAttribute,
    tex_coord: DynamicAttribute,
}

impl Shader<Ready> {
    pub fn position<'a>(&'a self) -> &'a DynamicAttribute { &self.position }

    pub fn fill_color<'a>(&'a self) -> &'a DynamicAttribute { &self.fill_color }

    pub fn tex_coord<'a>(&'a self) -> &'a DynamicAttribute { &self.tex_coord }
}
    
impl Shader<NotReady> {
    pub fn new() -> Shader<NotReady> {
        // Compile shaders.
        let vertex_shader = compile_shader(
                gl::VERTEX_SHADER,
                VERTEX_SHADER
            ).unwrap();
        let fragment_shader = compile_shader(
                gl::FRAGMENT_SHADER,
                FRAGMENT_SHADER
            ).unwrap();
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
       
        unsafe { 
            "out_color".with_c_str(
                |ptr| gl::BindFragDataLocation(program, 0, ptr)
            );
        }

        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
        };
        gl::LinkProgram(program);
        gl::UseProgram(program);

        let position = DynamicAttribute::xyz(
            program, "position", vao).unwrap();
        let fill_color = DynamicAttribute::rgb(
            program, "fill_color", vao).unwrap();
        let tex_coord = DynamicAttribute::uv(
            program, "tex_coord", vao).unwrap();
        Shader {
            vao: vao,
            program: program,
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            position: position,
            fill_color: fill_color,
            tex_coord: tex_coord,
        }
    }

    pub fn render(&self, gl: &mut Gl, f: |shader: &Shader<Ready>|) {
        gl.use_program(self.program);
        gl::BindVertexArray(self.vao);

        f(unsafe { &*(self as *const _ as *const Shader<Ready>) });

        gl::BindVertexArray(0);
    }
}

#[unsafe_destructor]
impl Drop for Shader<NotReady> {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteShader(self.fragment_shader);
    }
}

