
use std::cell::Cell;
use std::cmp;
use std::rand::{Rng, task_rng};

use array::*;
use chunk::{EMPTY_CHUNK, BiomeId, ChunkColumn, ChunkManager};
use super::Generator;
use super::diamondsquare::DiamondSquare;

// 10 -> 1024
// 9  -> 512
// 8  -> 256
static DEPTH: uint = 8;

pub struct TestWorldGenerator {
    player_pos: Option<[f32, ..3]>,
    player_pitch: Option<f32>,
    player_yaw: Option<f32>,
}

impl TestWorldGenerator {
    pub fn new() -> TestWorldGenerator {
        TestWorldGenerator {
            player_pos: None,
            player_pitch: None,
            player_yaw: None,
        }
    }
}

impl Generator for TestWorldGenerator {
    fn generate(&mut self, chunk_manager: &mut ChunkManager) {
        let h_octaves = vec![16.0, 16.0, 16.0, 8.0, 4.0, 1.0, 0.0, 0.0, 0.0];
        let mut height = DiamondSquare::new(DEPTH, h_octaves, 0.0);

        //let r_octaves = vec![32.0, 32.0, 16.0, 4.0, 1.0, 1.0, 2.0, 2.0, 1.0];
        //let mut rainfall = DiamondSquare::new(DEPTH, r_octaves, 32.0);

        let t_octaves = vec![32.0, 32.0, 16.0, 4.0, 1.0, 1.0, 2.0, 2.0, 1.0];
        let mut temp = DiamondSquare::new(DEPTH, t_octaves, 32.0);

        let mut rand = task_rng().gen();

        height.generate(&mut rand);
        //rainfall.generate(&mut rand);
        temp.generate(&mut rand);

        self.player_pos = Some([128.0, (height.get(128, 128) + 96.0).floor(), 128.0]);
        self.player_pitch = Some(0.0);
        self.player_yaw = Some(0.0);

        for cx in range(0u, 16) {
            for cz in range(0u, 16) {
                let mut chunks = Vec::from_elem(8, EMPTY_CHUNK);

                for x in range(0u, 16) {
                    for z in range(0u, 16) {
                        let hh = height.get(cx * 16 + x, cz * 16 + z);
                        let h = cmp::min(cmp::max((hh + 80.0).floor() as i32, 32), 112) as uint;
                        let t = temp.get((cx * 16 + x) / 2, (cz * 16 + z) / 2) - hh / 2.0;
                        //let r = rainfall.get((cx * 16 + x) / 2, (cz * 16 + z) / 2) - t / 2.0;
                        if rand.gen_range(92, 96) < h {
                            let hh1 = (hh + 80.0) + ((hh + 80.0) - 92.0) / 2.0;
                            let h1 = cmp::min(hh1 as uint, 128);
                            for y in range(0u, h1) {
                                chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0x1 << 4;
                            }
                            if t < -24.0 {
                                chunks.get_mut(h1 / 16).blocks[h1 % 16][z][x].value = 0x4F << 4;
                                chunks.get_mut((h1+1) / 16).blocks[(h1+1) % 16][z][x].value = 0x4E << 4;
                            }
                        } else {
                            let h1 = h - 5;
                            for y in range(0u, h1) {
                                chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0x1 << 4;
                            }
                            if h > 72 {
                                if t < 24.0 {
                                    for y in range(h1, h-2) {
                                        chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0x3 << 4;
                                    }
                                    chunks.get_mut((h-1) / 16).blocks[(h-1) % 16][z][x].value = 0x2 << 4;
                                    if t < -24.0 {
                                        chunks.get_mut(h / 16).blocks[h % 16][z][x].value = 0x4E << 4;
                                    }
                                } else {
                                    for y in range(h1, h) {
                                        chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0xC1;
                                    }
                                }
                            } else {
                                if t < 24.0 {
                                    for y in range(h1, h) {
                                        chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0xC0;
                                    }
                                } else {
                                    for y in range(h1, h) {
                                        chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0xC1;
                                    }
                                }
                            }
                        }
                        if h < 70 {
                            // FIXME: 0x023B is blue wool, not water.
                            if t > -24.0 {
                                for y in range(h, 70) {
                                    chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0x023B;
                                }
                            } else {
                                for y in range(h, 68) {
                                    chunks.get_mut(y / 16).blocks[y % 16][z][x].value = 0x023B;
                                }
                                chunks.get_mut(69 / 16).blocks[69 % 16][z][x].value = 0x04F0;
                            }
                        }
                    }
                }

                chunk_manager.add_chunk_column(cx as i32, cz as i32, ChunkColumn {
                    chunks: chunks,
                    buffers: Array::from_fn(|_| Cell::new(None)),
                    biomes: Array::from_fn(|_| -> [BiomeId, ..16] Array::from_fn(|_| {
                        BiomeId {
                            value: 0
                        }
                    }))
                });
            }
        }
    }

    fn player_pos(&self) -> Option<[f32, ..3]>
    {
        self.player_pos
    }

    fn player_pitch(&self) -> Option<f32>
    {
        self.player_pitch
    }

    fn player_yaw(&self) -> Option<f32>
    {
        self.player_yaw
    }
}
