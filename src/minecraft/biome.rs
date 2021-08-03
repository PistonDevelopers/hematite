use std::ops::Index;
use std::path::Path;

use crate::chunk::BiomeId;
use crate::minecraft::data;
use gfx_voxel::texture::ColorMap;

#[derive(Copy, Clone, Debug)]
pub struct Biome {
    pub name: &'static str,
    pub temperature: f32,
    pub humidity: f32,
    pub grass_color: [u8; 3],
    pub foliage_color: [u8; 3],
}

#[derive(Debug)]
pub struct Biomes {
    biomes: Box<[Option<Biome>; 256]>,
}

impl Biomes {
    #[must_use]
    pub fn load(assets: &Path) -> Biomes {
        let mut biomes = Box::new([None; 256]);

        let grass_colors = Path::new("minecraft/textures/colormap/grass.png");
        let grass_colors = ColorMap::from_path(&assets.join(&grass_colors)).unwrap();
        let foliage_colors = Path::new("minecraft/textures/colormap/foliage.png");
        let foliage_colors = ColorMap::from_path(&assets.join(foliage_colors)).unwrap();

        for (i, &biome) in data::BIOMES.iter().enumerate() {
            biomes[i] = biome.map(|(name, t, h)| Biome {
                name,
                temperature: t,
                humidity: h,
                grass_color: grass_colors.get(t, h),
                foliage_color: foliage_colors.get(t, h),
            });
        }

        Biomes { biomes }
    }
}

impl Index<BiomeId> for Biomes {
    type Output = Biome;

    fn index(&self, id: BiomeId) -> &Biome {
        self.biomes[id.value as usize].as_ref().unwrap()
    }
}
