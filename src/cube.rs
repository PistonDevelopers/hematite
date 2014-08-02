use vecmath::Vector3;

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

pub static QUADS: [[uint, ..4], ..6] = [
    [0u,4, 3, 7],   // left
    [5, 1, 6, 2],   // right
    [0, 1, 4, 5],   // bottom
    [2, 3, 6, 7],   // top
    [4, 5, 7, 6],   // front
    [1, 0, 2, 3]    // back
];

// Cube vertices.
pub static VERTICES: [Vector3, ..8] = [
    // This is the back surface
    [0.0f32,    0.0,        1.0], // 0
    [1.0,       0.0,        1.0], // 1
    [1.0,       1.0,        1.0], // 2
    [0.0,       1.0,        1.0], // 3

    // This is the front surface
    [0.0,       0.0,        0.0], // 4
    [1.0,       0.0,        0.0], // 5
    [1.0,       1.0,        0.0], // 6
    [0.0,       1.0,        0.0]  // 7
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
    pub fn vertices(self, x: f32, y: f32, z: f32) -> [Vector3, ..4] {
        let q = &QUADS[self as uint];
        let v = &VERTICES;
        [
            [x + v[q[0]][0], y + v[q[0]][1], z + v[q[0]][2]],
            [x + v[q[1]][0], y + v[q[1]][1], z + v[q[1]][2]],
            [x + v[q[2]][0], y + v[q[2]][1], z + v[q[2]][2]],
            [x + v[q[3]][0], y + v[q[3]][1], z + v[q[3]][2]]
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

