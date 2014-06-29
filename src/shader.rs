
use opengl_graphics::shader_utils::compile_shader;
use gl;
use gl::types::{
    GLfloat,
    GLint,
    GLsizei,
    GLsizeiptr,
    GLuint,
};

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


pub struct Shader {
    vertex_shader: GLuint,
    fragment_shader: GLuint,
    program: GLuint,
    position: GLuint,
    fill_color: GLuint,
    tex_coord: GLuint,
}

impl Shader {
    pub fn new() -> Shader {
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
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        gl::DeleteProgram(self.program);
        gl::DeleteShader(self.vertex_shader);
        gl::DeleteShader(self.fragment_shader);
    }
}

