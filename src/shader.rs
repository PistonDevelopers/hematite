
use opengl_graphics::shader_utils::compile_shader;
use opengl_graphics::Gl;
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

pub enum NotReady {}
pub enum Ready {}

pub struct TriList<'a> {
    pub texture_id: GLuint,
    pub vertices: &'a [f32],
    pub colors: &'a [f32],
    pub tex_coords: &'a [f32],
}

impl<'a> TriList<'a> {
    pub fn render(&'a self, shader: &Shader<Ready>) {
        let TriList {
            texture_id: texture_id, 
            vertices: vertices, 
            colors: colors, 
            tex_coords: tex_coords
        } = *self;
        gl::BindTexture(gl::TEXTURE_2D, texture_id);     

        let size_vertices: i32 = 3;
        let normalize_vertices = gl::FALSE;
        let vertices_byte_len = (
                vertices.len() * mem::size_of::<GLfloat>()
            ) as GLsizeiptr;
        // The data is tightly packed.
        let stride_vertices = 0;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.vbo[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices_byte_len,
                mem::transmute(&vertices[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.position,
                size_vertices,
                gl::FLOAT,
                normalize_vertices,
                stride_vertices,
                ptr::null()
            );
        }

        let size_fill_color = 3;
        let normalize_fill_color = gl::FALSE;
        let fill_colors_byte_len = (
                colors.len() * mem::size_of::<GLfloat>()
            ) as GLsizeiptr;
        // The data is tightly packed.
        let stride_fill_colors = 0;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.vbo[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                fill_colors_byte_len,
                mem::transmute(&colors[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.fill_color,
                size_fill_color,
                gl::FLOAT,
                normalize_fill_color,
                stride_fill_colors,
                ptr::null()
            );
        }

        let size_tex_coord = 2;
        let texture_coords_byte_len = (
                tex_coords.len() * mem::size_of::<GLfloat>()
            ) as GLsizeiptr;
        let normalize_texture_coords = gl::FALSE;
        // The data is tightly packed.
        let stride_texture_coords = 0;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.vbo[2]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                texture_coords_byte_len,
                mem::transmute(&tex_coords[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.tex_coord,
                size_tex_coord,
                gl::FLOAT,
                normalize_texture_coords,
                stride_texture_coords,
                ptr::null()
            );
        }

        // Draw front and back for testing.
        gl::CullFace(gl::FRONT_AND_BACK);

        let items: i32 = vertices.len() as i32 / size_vertices;
        gl::DrawArrays(gl::TRIANGLES, 0, items);
    }
}

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
    // TEST
    out_color = v_fill_color; // texture(s_texture, v_tex_coord) * v_fill_color;
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

