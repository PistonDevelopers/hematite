use gl::types::GLint;
use hgl;
use image;
use image::GenericImage;

pub enum MinecraftTexture {
    Grass,
}

impl MinecraftTexture {
    pub fn get_src_xy(&self) -> (i32, i32) {
        match *self {
            Grass => (0, 0),
        }
    }
}

pub struct Texture {
    tex: hgl::texture::Texture,
    width: u32,
    height: u32,
    unit_u: f32,
    unit_v: f32
}

impl Texture {
    /// Loads image by relative file name to the asset root.
    pub fn from_path(path: &Path) -> Result<Texture, String> {
        let img = match image::open(path) {
            Ok(img) => img,
            Err(e)  => return Err(format!("Could not load '{}': {}",
                                          path.filename_str().unwrap(), e)),
        };

        match img.color() {
            image::RGBA(8) => {},
            c => fail!("Unsupported color type {} in png", c),
        };

        let (width, height) = img.dimensions();

        let ii = hgl::texture::ImageInfo::new()
            .pixel_format(hgl::texture::pixel::RGBA).pixel_type(hgl::texture::pixel::UNSIGNED_BYTE)
            .width(width as GLint).height(height as GLint);

        let tex = hgl::Texture::new(hgl::texture::Texture2D, ii, img.raw_pixels().as_ptr());
        tex.gen_mipmaps();
        tex.filter(hgl::texture::Nearest);
        tex.wrap(hgl::texture::Repeat);

        Ok(Texture {
            tex: tex,
            width: width,
            height: height,
            unit_u: 16.0 / (width as f32),
            unit_v: 16.0 / (height as f32)
        })
    }

    pub fn square(&self, x: i32, y: i32) -> [[f32, ..2], ..4] {
        let (u1, v1) = (self.unit_u, self.unit_v);
        let (u, v) = (x as f32 * u1, y as f32 * v1);
        [
            [u, v],
            [u + u1, v],
            [u, v + v1],
            [u + u1, v + v1]
        ]
    }
}
