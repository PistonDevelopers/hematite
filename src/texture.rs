use gl::types::GLint;
use hgl;
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
    tex: hgl::texture::Texture,
    pub width: u32,
    pub height: u32
}

impl Texture {
    /// Loads image by relative file name to the asset root.
    pub fn from_path(path: &Path) -> Result<Texture, String> {
        Ok(Texture::from_rgba8(try!(load_rgba8(path))))
    }

    pub fn from_rgba8(img: ImageBuf<Rgba<u8>>) -> Texture {
        let (width, height) = img.dimensions();

        let ii = hgl::texture::ImageInfo::new()
            .pixel_format(hgl::texture::pixel::RGBA).pixel_type(hgl::texture::pixel::UNSIGNED_BYTE)
            .width(width as GLint).height(height as GLint);

        let tex = hgl::Texture::new(hgl::texture::Texture2D, ii,
                                    img.pixelbuf().as_ptr() as *const u8);
        tex.gen_mipmaps();
        tex.filter(hgl::texture::Nearest);
        tex.wrap(hgl::texture::Repeat);

        Texture {
            tex: tex,
            width: width,
            height: height
        }
    }
}
