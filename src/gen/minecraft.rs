
use std::cmp::max;
use std::io::File;

use array::*;
use chunk::ChunkManager;
use minecraft::nbt::Nbt;
use minecraft::region::Region;
use super::Generator;

pub struct MinecraftLoader {
    path: Path,
    player_pos: Option<[f32, ..3]>,
    player_pitch: Option<f32>,
    player_yaw: Option<f32>,
}

impl MinecraftLoader {
    pub fn new(path: Path) -> MinecraftLoader {
        MinecraftLoader {
            path: path,
            player_pos: None,
            player_pitch: None,
            player_yaw: None,
        }
    }
}

impl Generator for MinecraftLoader {
    fn generate(&mut self, chunk_manager: &mut ChunkManager) {
        let level = Nbt::from_gzip(File::open(&self.path.join("level.dat")).read_to_end().unwrap().as_slice()).unwrap();
        println!("{}", level);
        let player_pos = Array::from_iter(level["Data"]["Player"]["Pos"].as_double_list().unwrap().iter().map(|&x| x as f32));
        self.player_pos = Some(player_pos);
        let player_chunk = [player_pos.x(), player_pos.z()].map(|x| (x / 16.0).floor() as i32);
        let player_rot = level["Data"]["Player"]["Rotation"].as_float_list().unwrap();
        self.player_yaw = Some(player_rot[0]);
        self.player_pitch = Some(player_rot[1]);

        let [region_x, region_z] = player_chunk.map(|x| x >> 5);
        let region_file = self.path.join(format!("region/r.{}.{}.mca", region_x, region_z));
        let region = Region::open(&region_file);

        println!("Started loading chunks...");
        let [cx_base, cz_base] = player_chunk.map(|x| max(0, (x & 0x1f) - 8) as u8);
        for cz in range(cz_base, cz_base + 16) {
            for cx in range(cx_base, cx_base + 16) {
                match region.get_chunk_column(cx, cz) {
                    Some(column) => {
                        let [cx, cz] = [cx as i32 + region_x * 32, cz as i32 + region_z * 32];
                        chunk_manager.add_chunk_column(cx, cz, column)
                    }
                    None => {}
                }
            }
        }
        println!("Finished loading chunks.");
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
