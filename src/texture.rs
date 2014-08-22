use gfx;
use image;
use image::{GenericImage, ImageBuf, Pixel, Rgba};

fn load_rgba8(path: &Path) -> Result<ImageBuf<Rgba<u8>>, String> {
    match image::open(path) {
        Ok(image::ImageRgba8(img)) => Ok(img),
        Ok(image::ImageRgb8(img)) => {
            let (w, h) = img.dimensions();
            Ok(ImageBuf::from_fn(w, h, |x, y| img.get_pixel(x, y).to_rgba()))
        }
        Ok(img) => return Err(format!("Unsupported color type {} in '{}'",
                                      img.color(), path.display())),
        Err(e)  => return Err(format!("Could not load '{}': {}", path.display(), e))
    }
}

pub struct Texture {
    pub tex: gfx::TextureHandle,
    pub width: u32,
    pub height: u32
}

impl Texture {
    /// Loads image by relative file name to the asset root.
    pub fn from_path<D: gfx::Device>(path: &Path, d: &mut D) -> Result<Texture, String> {
        Ok(Texture::from_rgba8(try!(load_rgba8(path)), d))
    }

    pub fn from_rgba8<D: gfx::Device>(img: ImageBuf<Rgba<u8>>, d: &mut D) -> Texture {
        let (width, height) = img.dimensions();

        let mut ti = gfx::tex::TextureInfo::new();
        ti.width = width as u16;
        ti.height = height as u16;
        ti.kind = gfx::tex::Texture2D;
        ti.format = gfx::tex::RGBA8;

        let tex = d.create_texture(ti).unwrap();
        d.update_texture(&tex, &ti.to_image_info(), &img.into_vec()).unwrap();
        d.generate_mipmap(&tex);

        Texture {
            tex: tex,
            width: width,
            height: height
        }
    }
}

pub struct ColorMap {
    image: ImageBuf<Rgba<u8>>
}

impl ColorMap {
    pub fn from_path(path: &Path) -> Result<ColorMap, String> {
        let img = try!(load_rgba8(path));

        match img.dimensions() {
            (256, 256) => Ok(ColorMap {image: img}),
            (w, h) => Err(format!("ColorMap expected 256x256, found {}x{} in '{}'",
                                  w, h, path.display()))
        }
    }

    pub fn get(&self, x: f32, y: f32) -> [u8, ..3] {
        // Clamp to [0.0, 1.0].
        let x = x.max(0.0).min(1.0);
        let y = y.max(0.0).min(1.0);

        // Scale y from [0.0, 1.0] to [0.0, x], forming a triangle.
        let y = x * y;

        // Origin is in the bottom-right corner.
        let x = ((1.0 - x) * 255.0) as u8;
        let y = ((1.0 - y) * 255.0) as u8;

        let (r, g, b, _) = self.image.get_pixel(x as u32, y as u32).channels();
        [r, g, b]
    }
}
