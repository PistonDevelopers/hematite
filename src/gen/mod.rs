
use chunk::ChunkManager;

pub mod diamondsquare;
pub mod testgen;
pub mod minecraft;

pub trait Generator {
    fn generate(&mut self, &mut ChunkManager);

    fn player_pos(&self) -> Option<[f32, ..3]>;
    fn player_pitch(&self) -> Option<f32>;
    fn player_yaw(&self) -> Option<f32>;
}
