use std::ptr;
use std::mem;
use gl;
use gl::types::{
    GLfloat,
    GLsizeiptr,
};
use opengl_graphics::{
    Texture,
};
use shader::{
    Shader,
    Ready,
};
use graphics::ImageSize;

pub struct Quad<'a> {
    pub texture: &'a Texture,
    pub vertices: [f32, ..12],
    pub colors: [f32, ..12],
    pub tex_coords: [i32, ..8],
}

impl<'a> Quad<'a> {
    pub fn render(&'a self, shader: &Shader<Ready>) {
        let Quad {
            texture: texture, 
            vertices: vertices, 
            colors: colors, 
            tex_coords: tex_coords
        } = *self;
        gl::BindTexture(gl::TEXTURE_2D, texture.get_id());     

        let size_vertices: i32 = 3;
        let normalize_vertices = gl::FALSE;
        let vertices_byte_len = (
                vertices.len() * mem::size_of::<GLfloat>()
            ) as GLsizeiptr;
        // The data is tightly packed.
        let stride_vertices = 0;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.get_vbo()[0]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                vertices_byte_len,
                mem::transmute(&vertices[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.get_position(),
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
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.get_vbo()[1]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                fill_colors_byte_len,
                mem::transmute(&colors[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.get_fill_color(),
                size_fill_color,
                gl::FLOAT,
                normalize_fill_color,
                stride_fill_colors,
                ptr::null()
            );
        }

        let (w, h) = texture.get_size();
        let tex_coords = [
                tex_coords[0] as f32 / w as f32,
                tex_coords[1] as f32 / h as f32,
                tex_coords[2] as f32 / w as f32,
                tex_coords[3] as f32 / h as f32,
                tex_coords[4] as f32 / w as f32,
                tex_coords[5] as f32 / h as f32,
                tex_coords[6] as f32 / h as f32,
                tex_coords[7] as f32 / h as f32,
            ];
        let size_tex_coord = 2;
        let texture_coords_byte_len = (
                tex_coords.len() * mem::size_of::<GLfloat>()
            ) as GLsizeiptr;
        let normalize_texture_coords = gl::FALSE;
        // The data is tightly packed.
        let stride_texture_coords = 0;
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, shader.get_vbo()[2]);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                texture_coords_byte_len,
                mem::transmute(&tex_coords[0]),
                gl::DYNAMIC_DRAW
            );
            gl::VertexAttribPointer(
                shader.get_tex_coord(),
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
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, items);
    }
}
