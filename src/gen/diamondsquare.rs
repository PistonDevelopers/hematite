
use std::iter::range_step;
use std::rand::{IsaacRng, Rng};

pub struct DiamondSquare {
    depth: uint,
    dimension: uint,
    data: Vec<Vec<f32>>,
    base: f32,
    octaves: Vec<f32>,
    mod_bits: uint, // Used to modulo coordinates into the range [0, dimension) for wrapping.
}

impl DiamondSquare {
    pub fn new(depth: uint, octaves: Vec<f32>, base: f32) -> DiamondSquare {
        let dim = 1 << depth;
        let mut bits = 0;
        for i in range(0, depth) {
            bits <<= 1;
            bits |= 0b1;
        }
        DiamondSquare {
            depth: depth,
            dimension: dim,
            data: Vec::from_fn(dim, |_| Vec::from_fn(dim, |_| 0.0)),
            octaves: octaves,
            base: base,
            mod_bits: bits,
        }
    }

    pub fn generate(&mut self, rand: &mut IsaacRng) {
        *self.data.get_mut(0).get_mut(0) = if self.base == 0.0 { 0.0 } else { rand.gen_range(-self.base, self.base) };
        self.gen_recursive(rand, 0);
    }

    fn gen_recursive(&mut self, rand: &mut IsaacRng, depth: uint) {
        if depth > self.depth {
            return;
        }

        let step = self.dimension >> depth;
        let half_step = step >> 1;
        let octave = self.octaves[depth];

        for i in range_step(0, self.dimension, step) {
            for j in range_step(0, self.dimension, step) {
                let a = self.data[i][j];
                let b = self.data[(i+step) & self.mod_bits][j];
                let c = self.data[i][(j+step) & self.mod_bits];
                let d = self.data[(i+step) & self.mod_bits][(j+step) & self.mod_bits];

                let o = if octave == 0.0 { 0.0 } else { rand.gen_range(-octave, octave) };
                let mut_ref = self.data.get_mut((i+half_step) & self.mod_bits).get_mut((j+half_step) & self.mod_bits);
                *mut_ref = (a + b + c + d) / 4.0 + o;
            }
        }

        for i in range_step(0, self.dimension, step) {
            for j in range_step(0, self.dimension, step) {
                let a = self.data[i][j];
                let b = self.data[(i+step) & self.mod_bits][j];
                let c = self.data[i][(j+step) & self.mod_bits];
                let e = self.data[(i+half_step) & self.mod_bits][(j+half_step) & self.mod_bits];

                let f = (a + c + e + self.data[(i-half_step) & self.mod_bits][(j+half_step) & self.mod_bits]) / 4.0;
                let g = (a + b + e + self.data[(i+half_step) & self.mod_bits][(j-half_step) & self.mod_bits]) / 4.0;

                let o = if octave == 0.0 { 0.0 } else { rand.gen_range(-octave, octave) };
                *self.data.get_mut(i).get_mut((j+half_step) & self.mod_bits) = f + o;
                let o = if octave == 0.0 { 0.0 } else { rand.gen_range(-octave, octave) };
                *self.data.get_mut((i+half_step) & self.mod_bits).get_mut(j) = g + o;
            }
        }

        self.gen_recursive(rand, depth + 1);
    }

    #[inline]
    pub fn get(&self, i: uint, j: uint) -> f32 {
        self.data[i][j]
    }
}
