use std::collections::{Deque, RingBuf};
use time;

pub struct FPSCounter {
    last_second_frames: RingBuf<time::Timespec>
}

impl FPSCounter {
    pub fn new() -> FPSCounter {
        FPSCounter {
            last_second_frames: RingBuf::with_capacity(128)
        }
    }

    pub fn update(&mut self) -> uint {
        let now = time::now().to_timespec();
        let a_second_ago = time::Timespec::new(now.sec - 1, now.nsec);

        while self.last_second_frames.front().map_or(false, |t| *t < a_second_ago) {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push(now);
        self.last_second_frames.len()
    }
}
