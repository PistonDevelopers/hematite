use device::draw::CommandBuffer;
use gfx;
use gfx::Device;
use piston::image;
use piston::image::{GenericImage, ImageBuf, MutableRefImage, Pixel, Rgba, SubImage};
use std::collections::HashMap;
use std::collections::hashmap::{ Occupied, Vacant };
use std::mem;

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
    pub fn from_path<D: Device<C>, C: CommandBuffer>(path: &Path, d: &mut D)
                                                     -> Result<Texture, String> {
        Ok(Texture::from_rgba8(try!(load_rgba8(path)), d))
    }

    pub fn from_rgba8<D: Device<C>, C: CommandBuffer>(img: ImageBuf<Rgba<u8>>, d: &mut D)
                                                      -> Texture {
        let (width, height) = img.dimensions();

        let mut ti = gfx::tex::TextureInfo::new();
        ti.width = width as u16;
        ti.height = height as u16;
        ti.kind = gfx::tex::Texture2D;
        ti.format = gfx::tex::RGBA8;

        let tex = d.create_texture(ti).unwrap();
        d.update_texture(&tex, &ti.to_image_info(), img.into_vec().as_slice()).unwrap();
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

pub struct AtlasBuilder {
    image: ImageBuf<Rgba<u8>>,
    // Base path for loading tiles.
    path: Path,
    // Size of an individual tile.
    unit_width: u32,
    unit_height: u32,
    // Size of the entirely occupied square, in tiles.
    completed_tiles_size: u32,
    // Position in the current strip.
    position: u32,
    // Position cache for loaded tiles (in pixels).
    tile_positions: HashMap<String, (u32, u32)>,
    // Lowest-alpha cache for rectangles in the atlas.
    min_alpha_cache: HashMap<(u32, u32, u32, u32), u8>
}

impl AtlasBuilder {
    pub fn new(path: Path, unit_width: u32, unit_height: u32) -> AtlasBuilder {
        AtlasBuilder {
            image: ImageBuf::new(unit_width * 4, unit_height * 4),
            path: path,
            unit_width: unit_width,
            unit_height: unit_height,
            completed_tiles_size: 0,
            position: 0,
            tile_positions: HashMap::new(),
            min_alpha_cache: HashMap::new()
        }
    }

    pub fn load(&mut self, name: &str) -> (u32, u32) {
        match self.tile_positions.find_equiv(&name) {
            Some(pos) => return *pos,
            None => {}
        }

        let mut path = self.path.join(name);
        path.set_extension("png");
        let img = load_rgba8(&path).unwrap();

        let (iw, ih) = img.dimensions();
        assert!(iw == self.unit_width);
        assert!((ih % self.unit_height) == 0);
        if ih > self.unit_height {
            println!("ignoring {} extra frames in '{}'", (ih / self.unit_height) - 1, name);
        }

        let (uw, uh) = (self.unit_width, self.unit_height);
        let (w, h) = self.image.dimensions();
        let size = self.completed_tiles_size;

        // Expand the image buffer if necessary.
        if self.position == 0 && (uw * size >= w || uh * size >= h) {
            let old = mem::replace(&mut self.image, ImageBuf::new(w * 2, h * 2));
            let mut dest = SubImage::new(&mut self.image, 0, 0, w, h);
            for ((_, _, a), (_, _, b)) in dest.mut_pixels().zip(old.pixels()) {
                *a = b;
            }
        }

        let (x, y) = if self.position < size {
            (self.position, size)
        } else {
            (size, self.position - size)
        };

        self.position += 1;
        if self.position >= size * 2 + 1 {
            self.position = 0;
            self.completed_tiles_size += 1;
        }

        let mut dest = SubImage::new(&mut self.image, x * uw, y * uh, uw, uh);
        for ((_, _, a), (_, _, b)) in dest.mut_pixels().zip(img.pixels()) {
            *a = b;
        }

        *match self.tile_positions.entry(name.to_string()) {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.set((x * uw, y * uh))
        }
    }

    pub fn min_alpha(&mut self, x: u32, y: u32, w: u32, h: u32) -> u8 {
        match self.min_alpha_cache.find(&(x, y, w, h)) {
            Some(alpha) => return *alpha,
            None => {}
        }

        let tile = SubImage::new(&mut self.image, x, y, w, h);
        let min_alpha = tile.pixels().map(|(_, _, p)| p.alpha()).min().unwrap_or(0);
        self.min_alpha_cache.insert((x, y, w, h), min_alpha);
        min_alpha
    }

    pub fn complete<D: Device<C>, C: CommandBuffer>(self, d: &mut D) -> Texture {
        Texture::from_rgba8(self.image, d)
    }
}
