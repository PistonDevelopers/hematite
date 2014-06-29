use gl;
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

        unsafe { 
            shader.position().set(vertices);
            shader.fill_color().set(colors);
            
            // Compute texture coordinates in normalized uv.
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
            shader.tex_coord().set(tex_coords);
        }

        let size_vertices: i32 = 3;
        // Draw front and back for testing.
        gl::CullFace(gl::FRONT_AND_BACK);

        let items: i32 = vertices.len() as i32 / size_vertices;
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, items);
    }
}
