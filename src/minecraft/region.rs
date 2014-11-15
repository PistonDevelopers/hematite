use libc;
use std::cell::Cell;
use std::os;

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
    mmap: os::MemoryMap
}

impl Region {
    pub fn open(filename: &Path) -> Result<Region, String> {
        use std::mem::zeroed;

        unsafe {
            let fd = libc::open(
                    filename.as_str().unwrap().to_c_str().as_ptr(),
                    libc::consts::os::posix88::O_RDONLY,
                    libc::consts::os::posix88::S_IREAD
                );
            // If a negative file descriptor is returned,
            // an error occured when attempting to open the file.
            if fd < 0 {
                return Err(format!("An error occured when opening `{}`: {}", filename.as_str(), fd));
            }
            let mut stat = zeroed();
            libc::fstat(fd, &mut stat);
            let min_len = stat.st_size as uint;
            let options = [
                os::MapFd(fd),
                os::MapReadable
            ];

            let res = Region {
                mmap: os::MemoryMap::new(min_len, options).unwrap()
            };
            libc::close(fd);
            Ok(res)
        }
    }

    fn as_slice<'a>(&'a self) -> &'a [u8] {
        use std::mem;
        use std::raw::Slice;
        let slice = Slice {
                data: self.mmap.data() as *const u8,
                len: self.mmap.len()
            };

        unsafe { mem::transmute(slice) }
    }

    pub fn get_chunk_column(&self, x: u8, z: u8) -> Option<ChunkColumn> {
        let locations = self.as_slice().slice_to(4096);
        let i = 4 * ((x % 32) as uint + (z % 32) as uint * 32);
        let start = (locations[i] as uint << 16)
                  | (locations[i + 1] as uint << 8)
                  | locations[i + 2] as uint;
        let num = locations[i + 3] as uint;
        if start == 0 || num == 0 { return None; }

        let sectors = self.as_slice().slice(start * 4096, (start + num) * 4096);
        let len = (sectors[0] as uint << 24)
                | (sectors[1] as uint << 16)
                | (sectors[2] as uint << 8)
                | sectors[3] as uint;
        let nbt = match sectors[4] {
            1 => Nbt::from_gzip(sectors.slice(5, 4 + len)),
            2 => Nbt::from_zlib(sectors.slice(5, 4 + len)),
            c => panic!("unknown region chunk compression method {}", c)
        };

        let mut c = nbt.unwrap().into_compound().unwrap();
        let mut level = c.pop_equiv("Level").unwrap().into_compound().unwrap();
        let mut chunks = Vec::new();
        for chunk in level.pop_equiv("Sections")
            .unwrap().into_compound_list().unwrap().into_iter() {

            let y = chunk.find_equiv("Y")
                .unwrap().as_byte().unwrap();
            let blocks = chunk.find_equiv("Blocks")
                .unwrap().as_bytearray().unwrap();
            let blocks_top = chunk.find_equiv("Add")
                .and_then(|x| x.as_bytearray());
            let blocks_data = chunk.find_equiv("Data")
                .unwrap().as_bytearray().unwrap();
            let block_light = chunk.find_equiv("BlockLight")
                .unwrap().as_bytearray().unwrap();
            let sky_light = chunk.find_equiv("SkyLight")
                .unwrap().as_bytearray().unwrap();

            fn array_16x16x16<T>(
                f: |uint, uint, uint| -> T
            ) -> [[[T, ..SIZE], ..SIZE], ..SIZE] {
                Array::from_fn(|y| -> [[T, ..SIZE], ..SIZE]
                    Array::from_fn(|z| -> [T, ..16]
                        Array::from_fn(|x| f(x, y, z))
                    )
                )
            }

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
                        value: (blocks[i] as u16 << 4)
                             | (top as u16 << 12)
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
            let len = chunks.len();
            if y as uint >= len {
                chunks.grow(y as uint - len + 1, *EMPTY_CHUNK);
            }
            chunks[y as uint] = chunk;
        }
        let biomes = level.find_equiv("Biomes")
            .unwrap().as_bytearray().unwrap();
        Some(ChunkColumn {
            chunks: chunks,
            buffers: Array::from_fn(|_| Cell::new(None)),
            biomes: Array::from_fn(|z| -> [BiomeId, ..SIZE] Array::from_fn(|x| {
                BiomeId {
                    value: biomes[z * SIZE + x]
                }
            }))
        })
    }
}
