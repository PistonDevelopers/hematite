use vecmath::Vector3;

/*
        3  ---------  2
          /       / |
         /  up   /  |
     6  -------- 7  | 1
       |        |  /
west   |  south | /  east
       |        |/
     5  -------- 4

*/

/*
        7  ---------  6
          /       / |
         /  up   /  |
     2  -------- 3  | 5
       |        |  /
east   |  north | /  west
       |        |/
     1  -------- 0

*/

// Cube faces (clockwise).
pub static QUADS: [[uint, ..4], ..6] = [
    [1, 0, 5, 4], // down
    [7, 6, 3, 2], // up
    [0, 1, 2, 3], // north
    [4, 5, 6, 7], // south
    [5, 0, 3, 6], // west
    [1, 4, 7, 2]  // east
];

// Cube vertices.
pub static VERTICES: [Vector3, ..8] = [
    // This is the north surface
    [0.0, 0.0, 0.0], // 0
    [1.0, 0.0, 0.0], // 1
    [1.0, 1.0, 0.0], // 2
    [0.0, 1.0, 0.0], // 3

    // This is the south surface
    [1.0, 0.0, 1.0], // 4
    [0.0, 0.0, 1.0], // 5
    [0.0, 1.0, 1.0], // 6
    [1.0, 1.0, 1.0]  // 7
];

#[repr(uint)]
#[deriving(PartialEq, Eq, FromPrimitive, Show)]
pub enum Face {
    Down = 0,
    Up = 1,
    North = 2,
    South = 3,
    West = 4,
    East = 5
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

