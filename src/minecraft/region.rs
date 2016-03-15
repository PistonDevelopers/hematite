use std::cell::RefCell;
use std::io;
use std::path::Path;
use gfx;
use memmap::{Mmap, Protection};

use array::*;
use chunk::{
    BiomeId,
    BlockState,
    Chunk,
    ChunkColumn,
    EMPTY_CHUNK,
    LightLevel,
    SIZE
};
use minecraft::nbt::Nbt;

pub struct Region {
    mmap: Mmap,
}

fn array_16x16x16<T, F>(mut f: F) -> [[[T; SIZE]; SIZE]; SIZE]
    where F: FnMut(usize, usize, usize) -> T
{
    Array::from_fn(|y| -> [[T; SIZE]; SIZE] {
        Array::from_fn(|z| -> [T; 16] {
            Array::from_fn(|x| f(x, y, z))
        })
    })
}

impl Region {
    pub fn open(filename: &Path) -> io::Result<Region> {
        let mmap = try!(Mmap::open_path(filename, Protection::Read));
        Ok(Region{mmap: mmap})
    }

    fn as_slice(&self) -> &[u8] {
        unsafe {
            self.mmap.as_slice()
        }
    }

    pub fn get_chunk_column<R: gfx::Resources>(&self, x: u8, z: u8)
                            -> Option<ChunkColumn<R>> {
        let locations = &self.as_slice()[..4096];
        let i = 4 * ((x % 32) as usize + (z % 32) as usize * 32);
        let start = ((locations[i] as usize) << 16)
                  | ((locations[i + 1] as usize) << 8)
                  | (locations[i + 2] as usize);
        let num = locations[i + 3] as usize;
        if start == 0 || num == 0 { return None; }
        let sectors = &self.as_slice()[start * 4096 .. (start + num) * 4096];
        let len = ((sectors[0] as usize) << 24)
                | ((sectors[1] as usize) << 16)
                | ((sectors[2] as usize) << 8)
                | (sectors[3] as usize);
        let nbt = match sectors[4] {
            1 => Nbt::from_gzip(&sectors[5 .. 4 + len]),
            2 => Nbt::from_zlib(&sectors[5 .. 4 + len]),
            c => panic!("unknown region chunk compression method {}", c)
        };

        let mut c = nbt.unwrap().into_compound().unwrap();
        let mut level = c.remove("Level").unwrap().into_compound().unwrap();
        let mut chunks = Vec::new();
        for chunk in level.remove("Sections")
            .unwrap().into_compound_list().unwrap().into_iter() {

            let y = chunk.get("Y")
                .unwrap().as_byte().unwrap();
            let blocks = chunk.get("Blocks")
                .unwrap().as_bytearray().unwrap();
            let blocks_top = chunk.get("Add")
                .and_then(|x| x.as_bytearray());
            let blocks_data = chunk.get("Data")
                .unwrap().as_bytearray().unwrap();
            let block_light = chunk.get("BlockLight")
                .unwrap().as_bytearray().unwrap();
            let sky_light = chunk.get("SkyLight")
                .unwrap().as_bytearray().unwrap();

            let chunk = Chunk {
                blocks: array_16x16x16(|x, y, z| {
                    let i = (y * SIZE + z) * SIZE + x;
                    let top = match blocks_top {
                        Some(blocks_top) => {
                            (blocks_top[i >> 1] >> ((i & 1) * 4)) & 0x0f
                        }
                        None => 0
                    };
                    let data = (blocks_data[i >> 1] >> ((i & 1) * 4)) & 0x0f;
                    BlockState {
                        value: ((blocks[i] as u16) << 4)
                             | ((top as u16) << 12)
                             | (data as u16)
                    }
                }),
                light_levels: array_16x16x16(|x, y, z| {
                    let i = (y * 16 + z) * 16 + x;
                    let block = (block_light[i >> 1] >> ((i & 1) * 4)) & 0x0f;
                    let sky = (sky_light[i >> 1] >> ((i & 1) * 4)) & 0x0f;
                    LightLevel {
                        value: block | (sky << 4)
                    }
                }),
            };
            while chunks.len() <= y as usize {
                chunks.push(*EMPTY_CHUNK);
            }
            chunks[y as usize] = chunk;
        }
        let biomes = level.get("Biomes")
            .unwrap().as_bytearray().unwrap();
        Some(ChunkColumn {
            chunks: chunks,
            buffers: Array::from_fn(|_| RefCell::new(None)),
            biomes: Array::from_fn(|z| -> [BiomeId; SIZE] {
                Array::from_fn(|x| {
                    BiomeId {
                        value: biomes[z * SIZE + x]
                    }
                })
            })
        })
    }
}
