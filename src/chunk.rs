use std::cell::Cell;
use std::collections::HashMap;

use array::*;
use shader::Buffer;

pub struct BlockState {
    pub value: u16
}

pub const EMPTY_BLOCK: BlockState = BlockState { value: 0 };

pub struct BiomeId {
    pub value: u8
}

pub struct LightLevel {
    pub value: u8
}

impl LightLevel {
    pub fn block_light(self) -> u8 {
        self.value & 0xf
    }
    pub fn sky_light(self) -> u8 {
        self.value >> 4
    }
}

pub const SIZE: uint = 16;

/// A chunk of SIZE x SIZE x SIZE blocks, in YZX order.
pub struct Chunk {
    pub blocks: [[[BlockState, ..SIZE], ..SIZE], ..SIZE],
    pub light_levels: [[[LightLevel, ..SIZE], ..SIZE], ..SIZE]
}

impl Clone for Chunk {
    fn clone(&self) -> Chunk {
        *self
    }
}

// TODO: Change to const pointer.
pub const EMPTY_CHUNK: &'static Chunk = &Chunk {
    blocks: [[[EMPTY_BLOCK, ..SIZE], ..SIZE], ..SIZE],
    light_levels: [[[LightLevel {value: 0xf0}, ..SIZE], ..SIZE], ..SIZE]
};

pub struct ChunkColumn {
    pub chunks: Vec<Chunk>,
    pub buffers: [Cell<Option<Buffer>>, ..SIZE],
    pub biomes: [[BiomeId, ..SIZE], ..SIZE]
}

pub struct ChunkManager {
    chunk_columns: HashMap<(i32, i32), ChunkColumn>
}

impl ChunkManager {
    pub fn new() -> ChunkManager {
        ChunkManager {
            chunk_columns: HashMap::new()
        }
    }

    pub fn add_chunk_column(&mut self, x: i32, z: i32, c: ChunkColumn) {
        self.chunk_columns.insert((x, z), c);
    }

    pub fn each_chunk_and_neighbors<'a>(
        &'a self,
        f: |
            coords: [i32, ..3],
            buffer: &'a Cell<Option<Buffer>>,
            chunks: [[[&'a Chunk, ..3], ..3], ..3],
            biomes: [[Option<&'a [[BiomeId, ..SIZE], ..SIZE]>, ..3], ..3]
        |
    ) {
        for &(x, z) in self.chunk_columns.keys() {
            let columns = [-1, 0, 1].map(
                    |dz| [-1, 0, 1].map(
                        |dx| self.chunk_columns.get(&(x + dx, z + dz))
                    )
                );
            let central = columns[1][1].unwrap();
            for y in range(0, central.chunks.len()) {
                let chunks = [-1, 0, 1].map(|dy| {
                    let y = y as i32 + dy;
                    columns.map(
                        |cz| cz.map(
                            |cx| cx.and_then(
                                |c| c.chunks.as_slice().get(y as uint)
                            ).unwrap_or(EMPTY_CHUNK)
                        )
                    )
                });
                f([x, y as i32, z], &central.buffers[y], chunks,
                  columns.map(|cz| cz.map(|cx| cx.map(|c| &c.biomes))))
            }
        }
    }

    pub fn each_chunk(
        &self,
        f: |x: i32, y: i32, z: i32, c: &Chunk, b: Option<Buffer>|
    ) {
        for (&(x, z), c) in self.chunk_columns.iter() {
            for (y, (c, b)) in c.chunks.iter()
                .zip(c.buffers.iter()).enumerate() {
                
                f(x, y as i32, z, c, b.get())
            }
        }
    }
}
