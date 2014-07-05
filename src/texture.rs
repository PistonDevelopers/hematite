use quad::Quad;
use opengl_graphics::{Texture};

pub enum MinecraftTexture {
    Grass,
}

impl MinecraftTexture {
    pub fn get_src_xy(&self) -> (i32, i32) {
        match *self {
            Grass => (0, 0),
        }
    }

    pub fn to_quad<'a>(&self, texture: &'a Texture) -> Quad<'a> {
        let (src_x, src_y) = self.get_src_xy();
        Quad {
            texture: texture,
            vertices: [
                0.0, 0.0, 0.0,
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                1.0, 1.0, 0.0,
            ],
            colors: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                1.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ],
            tex_coords: [
                src_x, src_y,
                src_x + 16, src_y,
                src_x, src_y + 16,
                src_x + 16, src_y + 16,
            ],
        }
    }    
}

