
use opengl_graphics::shader_utils::compile_shader;
use opengl_graphics::{
    Gl,
    Texture,
};
use gl;
use gl::types::{
    GLfloat,
    GLint,
    GLsizei,
    GLsizeiptr,
    GLuint,
};
use std::ptr;
use std::mem;
use graphics::ImageSize;

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
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,
    position: GLuint,
    fill_color: GLuint,
    tex_coord: GLuint,
    vbo: [GLuint, ..3],
}

impl Shader<Ready> {
    #[inline(always)]
    pub fn get_vbo<'a>(&'a self) -> &'a [GLuint, ..3] { &(self.vbo) }

    #[inline(always)]
    pub fn get_position(&self) -> GLuint { self.position }

    #[inline(always)]
    pub fn get_fill_color(&self) -> GLuint { self.fill_color }

    #[inline(always)]
    pub fn get_tex_coord(&self) -> GLuint { self.tex_coord }
}

impl Shader<NotReady> {
    pub fn new() -> Shader<NotReady> {
        let mut vbo: [GLuint, ..3] = [0, ..3];
        unsafe {
            gl::GenBuffers(3, vbo.as_mut_ptr());
            gl::GenVertexArrays(3, vbo.as_mut_ptr());
        }

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

        gl::LinkProgram(program);
        gl::UseProgram(program);

        unsafe {
            let position = "position".with_c_str(
                |ptr| gl::GetAttribLocation(program, ptr)
            );
            let fill_color = "fill_color".with_c_str(
                |ptr| gl::GetAttribLocation(program, ptr)
            );
            let tex_coord = "tex_coord".with_c_str(
                |ptr| gl::GetAttribLocation(program, ptr)
            );
            Shader {
                program: program,
                vertex_shader: vertex_shader,
                fragment_shader: fragment_shader,
                position: position as GLuint,
                fill_color: fill_color as GLuint,
                tex_coord: tex_coord as GLuint,
                vbo: vbo,
            }
        }
    }

    pub fn render(&self, gl: &mut Gl, f: |shader: &Shader<Ready>|) {
        gl.use_program(self.program);
        gl::BindVertexArray(self.vbo[0]);
        gl::BindVertexArray(self.vbo[1]);
        gl::BindVertexArray(self.vbo[2]);

        gl::EnableVertexAttribArray(self.position);
        gl::EnableVertexAttribArray(self.fill_color);
        gl::EnableVertexAttribArray(self.tex_coord);

        f(unsafe {mem::transmute(self)});

        gl::DisableVertexAttribArray(self.vbo[0]);
        gl::DisableVertexAttribArray(self.vbo[1]);
        gl::DisableVertexAttribArray(self.vbo[2]); 
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

