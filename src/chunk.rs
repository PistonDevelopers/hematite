use std::cell::RefCell;
use std::collections::HashMap;

use array::*;
use shader::Vertex;
use gfx;

#[derive(Copy, Clone)]
pub struct BlockState {
    pub value: u16
}

pub const EMPTY_BLOCK: BlockState = BlockState { value: 0 };

#[derive(Copy, Clone)]
pub struct BiomeId {
    pub value: u8
}

#[derive(Copy, Clone)]
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

pub const SIZE: usize = 16;

/// A chunk of SIZE x SIZE x SIZE blocks, in YZX order.
#[derive(Copy, Clone)]
pub struct Chunk {
    pub blocks: [[[BlockState; SIZE]; SIZE]; SIZE],
    pub light_levels: [[[LightLevel; SIZE]; SIZE]; SIZE]
}

// TODO: Change to const pointer.
pub const EMPTY_CHUNK: &'static Chunk = &Chunk {
    blocks: [[[EMPTY_BLOCK; SIZE]; SIZE]; SIZE],
    light_levels: [[[LightLevel {value: 0xf0}; SIZE]; SIZE]; SIZE]
};

pub struct ChunkColumn<R: gfx::Resources> {
    pub chunks: Vec<Chunk>,
    pub buffers: [RefCell<Option<gfx::handle::Buffer<R, Vertex>>>; SIZE],
    pub biomes: [[BiomeId; SIZE]; SIZE]
}

pub struct ChunkManager<R: gfx::Resources> {
    chunk_columns: HashMap<(i32, i32), ChunkColumn<R>>
}

impl<R: gfx::Resources> ChunkManager<R> {
    pub fn new() -> ChunkManager<R> {
        ChunkManager {
            chunk_columns: HashMap::new()
        }
    }

    pub fn add_chunk_column(&mut self, x: i32, z: i32, c: ChunkColumn<R>) {
        self.chunk_columns.insert((x, z), c);
    }

    pub fn each_chunk_and_neighbors<'a, F>(&'a self, mut f: F)
        where F: FnMut(/*coords:*/ [i32; 3],
                       /*buffer:*/ &'a RefCell<Option<gfx::handle::Buffer<R, Vertex>>>,
                       /*chunks:*/ [[[&'a Chunk; 3]; 3]; 3],
                       /*biomes:*/ [[Option<&'a [[BiomeId; SIZE]; SIZE]>; 3]; 3])

    {
        for &(x, z) in self.chunk_columns.keys() {
            let columns = [-1, 0, 1].map(
                    |dz| [-1, 0, 1].map(
                        |dx| self.chunk_columns.get(&(x + dx, z + dz))
                    )
                );
            let central = columns[1][1].unwrap();
            for y in 0..central.chunks.len() {
                let chunks = [-1, 0, 1].map(|dy| {
                    let y = y as i32 + dy;
                    columns.map(
                        |cz| cz.map(
                            |cx| cx.and_then(
                                |c| c.chunks[..].get(y as usize)
                            ).unwrap_or(EMPTY_CHUNK)
                        )
                    )
                });
                f([x, y as i32, z], &central.buffers[y], chunks,
                  columns.map(|cz| cz.map(|cx| cx.map(|c| &c.biomes))))
            }
        }
    }

    pub fn each_chunk<F>(&self, mut f: F)
        where F: FnMut(/*x:*/ i32, /*y:*/ i32, /*z:*/ i32, /*c:*/ &Chunk, 
            /*b:*/ &RefCell<Option<gfx::handle::Buffer<R, Vertex>>>)
    {
        for (&(x, z), c) in self.chunk_columns.iter() {
            for (y, (c, b)) in c.chunks.iter()
                .zip(c.buffers.iter()).enumerate() {

                f(x, y as i32, z, c, b)
            }
        }
    }
}
