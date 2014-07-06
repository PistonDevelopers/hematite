
/*
        3  ---------  2
          /       / |
         /  top  /  |
     7  -------- 6  | 1
       |        |  /
left   |  front | /  right
       |        |/
     4  -------- 5

*/

pub static QUADS: [uint, ..24] = [
    0u, 4, 3, 7,    // left
    5, 1, 6, 2,     // right
    0, 1, 4, 5,     // bottom
    3, 2, 7, 6,     // top
    4, 5, 7, 6,     // front
    1, 0, 2, 3,     // back
];

// Cube vertices.
pub static VERTICES: [f32, ..24] = [
    // This is the back surface
    -1.0f32,    -1.0,       1.0, // 0
     1.0,       -1.0,       1.0, // 1
     1.0,        1.0,       1.0, // 2
    -1.0,        1.0,       1.0, // 3

    // This is the front surface
    -1.0,       -1.0,      -1.0, // 4
     1.0,       -1.0,      -1.0, // 5
     1.0,        1.0,      -1.0, // 6
    -1.0,        1.0,      -1.0  // 7
];


#[repr(uint)]
#[deriving(FromPrimitive)]
pub enum Face {
    Left = 0,
    Right = 1,
    Bottom = 2,
    Top = 3,
    Front = 4,
    Back = 5,
}

impl Face {
    #[inline(always)]
    fn ind(self, vertex: uint, dim: uint) -> uint {
        QUADS[self as uint * 4 + vertex] * 3 + dim
    }

    pub fn vertices(self, scale: f32) -> [f32, ..12] {
        [
            VERTICES[self.ind(0, 0)] * scale, 
            VERTICES[self.ind(0, 1)] * scale, 
            VERTICES[self.ind(0, 2)] * scale,
    
            VERTICES[self.ind(1, 0)] * scale, 
            VERTICES[self.ind(1, 1)] * scale, 
            VERTICES[self.ind(1, 2)] * scale,
    
            VERTICES[self.ind(2, 0)] * scale, 
            VERTICES[self.ind(2, 1)] * scale, 
            VERTICES[self.ind(2, 2)] * scale,
    
            VERTICES[self.ind(3, 0)] * scale, 
            VERTICES[self.ind(3, 1)] * scale, 
            VERTICES[self.ind(3, 2)] * scale,
        ]
    }
}

pub struct FaceIterator {
    face: uint,
}

impl FaceIterator {
    pub fn new() -> FaceIterator {
        FaceIterator {
            face: 0
        }
    }
}

impl Iterator<Face> for FaceIterator {
    fn next(&mut self) -> Option<Face> {
        match self.face {
            x if x < 6 => {
                self.face += 1;
                FromPrimitive::from_uint(x)
            },
            _ => None
        }
    }
}

