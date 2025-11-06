use crate::color::{Color, ParsingErr};
use crate::cube::{Cube, NUM_FACES, RefCube};
use crate::face::Coord;
use crate::turn::{
    ALL_DIRS, ALL_FACES, CubeFace, Direction, NUM_DIRS, Turn, Turns,
};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum CornerIndex {
    #[default]
    DBL = 0,
    DLF = 1,
    DRB = 2,
    DFR = 3,
    ULB = 4,
    UFL = 5,
    UBR = 6,
    URF = 7,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum Orientation {
    #[default]
    Nothing = 0,
    Clockwise = 1,
    Counterclockwise = 2,
}

pub const NUM_ORIENTATIONS: usize = 3;
pub const ALL_ORIENTATIONS: [Orientation; NUM_ORIENTATIONS] = [
    Orientation::Nothing,
    Orientation::Clockwise,
    Orientation::Counterclockwise,
];

impl Orientation {
    fn compose(&self, orientation: Orientation) -> Orientation {
        match ((*self as usize) + (orientation as usize)) % 3 {
            0 => Orientation::Nothing,
            1 => Orientation::Clockwise,
            2 => Orientation::Counterclockwise,
            _ => panic!("This should never happen"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Corner {
    pub index: CornerIndex,
    pub orientation: Orientation,
}

impl Corner {
    fn new(index: CornerIndex) -> Self {
        Corner {
            index,
            orientation: Orientation::Nothing,
        }
    }
}

pub const NUM_CORNERS: usize = 8;
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Cube2 {
    pub corners: [Corner; NUM_CORNERS],
    color_map: [Color; NUM_FACES],
}

const NUM_CORNER_FACES: usize = 3;
pub const CORNER_INDICES: [CornerIndex; NUM_CORNERS] = [
    CornerIndex::DBL,
    CornerIndex::DLF,
    CornerIndex::DRB,
    CornerIndex::DFR,
    CornerIndex::ULB,
    CornerIndex::UFL,
    CornerIndex::UBR,
    CornerIndex::URF,
];
const CUBE_FACES: [[CubeFace; NUM_CORNER_FACES]; NUM_CORNERS] = [
    [CubeFace::Down, CubeFace::Back, CubeFace::Left],
    [CubeFace::Down, CubeFace::Left, CubeFace::Front],
    [CubeFace::Down, CubeFace::Right, CubeFace::Back],
    [CubeFace::Down, CubeFace::Front, CubeFace::Right],
    [CubeFace::Up, CubeFace::Left, CubeFace::Back],
    [CubeFace::Up, CubeFace::Front, CubeFace::Left],
    [CubeFace::Up, CubeFace::Back, CubeFace::Right],
    [CubeFace::Up, CubeFace::Right, CubeFace::Front],
];
const COORDS: [[Coord; NUM_CORNER_FACES]; NUM_CORNERS] = [
    [Coord::new(1, 0), Coord::new(1, 1), Coord::new(1, 0)],
    [Coord::new(0, 0), Coord::new(1, 1), Coord::new(1, 0)],
    [Coord::new(1, 1), Coord::new(1, 1), Coord::new(1, 0)],
    [Coord::new(0, 1), Coord::new(1, 1), Coord::new(1, 0)],
    [Coord::new(0, 0), Coord::new(0, 0), Coord::new(0, 1)],
    [Coord::new(1, 0), Coord::new(0, 0), Coord::new(0, 1)],
    [Coord::new(0, 1), Coord::new(0, 0), Coord::new(0, 1)],
    [Coord::new(1, 1), Coord::new(0, 0), Coord::new(0, 1)],
];
const CORNER_COLORS: [[Color; NUM_CORNER_FACES]; NUM_CORNERS] = [
    [Color::Yellow, Color::Blue, Color::Orange],
    [Color::Yellow, Color::Orange, Color::Green],
    [Color::Yellow, Color::Red, Color::Blue],
    [Color::Yellow, Color::Green, Color::Red],
    [Color::White, Color::Orange, Color::Blue],
    [Color::White, Color::Green, Color::Orange],
    [Color::White, Color::Blue, Color::Red],
    [Color::White, Color::Red, Color::Green],
];
const NUM_TURNS: usize = 18;
const NUM_CORNERS_PER_TURN: usize = 4;

impl Cube2 {
    fn apply_orientation(
        orientation: Orientation,
        colors: &[Color; NUM_CORNER_FACES],
    ) -> [Color; NUM_CORNER_FACES] {
        match orientation {
            Orientation::Nothing => colors.clone(),
            Orientation::Clockwise => [colors[1], colors[2], colors[0]],
            Orientation::Counterclockwise => [colors[2], colors[0], colors[1]],
        }
    }

    // Returns the position and the orientation at which we found `colors`
    fn find_corner(
        ref_cube: &RefCube<2>,
        colors: &[Color; NUM_CORNER_FACES],
    ) -> Result<(usize, Orientation), ParsingErr> {
        // Go through each corner of the cube
        for ((faces, coords), index) in
            CUBE_FACES.iter().zip(COORDS).zip(CORNER_INDICES)
        {
            // Extract the color of this corner of the cube
            let mut cube_colors = [Color::White; 3];
            for (i, (face, coord)) in faces.iter().zip(coords).enumerate() {
                let color = ref_cube.0[*face as usize].at(&coord);
                cube_colors[i] = color;
            }

            // Try out each orientation of the colors of this corner, and see
            // which orientation matches the caller wants
            for orientation in [
                Orientation::Nothing,
                Orientation::Clockwise,
                Orientation::Counterclockwise,
            ] {
                let colors = Cube2::apply_orientation(orientation, colors);
                if cube_colors == colors {
                    return Ok((index as usize, orientation));
                }
            }
        }
        // TODO: change the Cube trait to return any type of parsing error
        // instead
        Err(ParsingErr::InvalidColor)
    }

    fn apply_turn_ref(&mut self, turn: &Turn) {
        let mut ref_cube: RefCube<2> = self.clone().into();
        ref_cube.apply_turn(turn);
        *self = Cube2::parse_str(&ref_cube.to_color_string()).unwrap();
    }

    fn apply_turns_ref(&mut self, turns: &Turns) {
        for turn in turns.iter() {
            self.apply_turn_ref(turn);
        }
    }

    pub fn gen_turn_table() {
        // 6 faces, 3 directions
        let mut table = [[(0, Orientation::Nothing, 0); 4]; NUM_TURNS];
        for turn in Turn::all_turns::<2>().iter() {
            let mut cube = Cube2::new();
            cube.apply_turn_ref(turn);
            let mut permutation_map = HashMap::new();
            let mut orientation_map = HashMap::new();
            let mut indices = [0; 4];
            let mut i = 0;
            for (index, corner) in CORNER_INDICES.iter().zip(cube.corners) {
                if *index != corner.index {
                    let cur_index = corner.index as usize;
                    indices[i] = cur_index;
                    i += 1;
                    permutation_map.insert(cur_index, *index as usize);
                    orientation_map.insert(cur_index, corner.orientation);
                }
            }
            assert_eq!(permutation_map.len(), 4);
            assert_eq!(orientation_map.len(), 4);

            let turn_idx: usize = turn.clone().into();
            let entries = &mut table[turn_idx];
            for (i, cur_index) in indices.iter().enumerate() {
                entries[i] = (
                    *cur_index,
                    orientation_map[cur_index],
                    permutation_map[cur_index],
                );
            }
        }
        println!("Table:\n{:?}", table);
    }
}

impl From<Turn> for usize {
    fn from(turn: Turn) -> usize {
        assert_eq!(turn.num_layers, 1);
        (turn.dir as usize) * NUM_FACES + (turn.face as usize)
    }
}

impl From<usize> for Turn {
    fn from(turn: usize) -> Turn {
        let face = ALL_FACES[turn % NUM_FACES];
        let dir = turn / NUM_FACES;
        assert!(dir < NUM_DIRS);
        let dir = ALL_DIRS[dir];
        Turn {
            face,
            dir,
            num_layers: 1,
        }
    }
}

const TURN_TABLE: [[(usize, Orientation, usize); NUM_CORNERS_PER_TURN];
    NUM_TURNS] = [
    [
        (5, Orientation::Nothing, 4),
        (7, Orientation::Nothing, 5),
        (4, Orientation::Nothing, 6),
        (6, Orientation::Nothing, 7),
    ],
    [
        (1, Orientation::Counterclockwise, 0),
        (5, Orientation::Clockwise, 1),
        (0, Orientation::Clockwise, 4),
        (4, Orientation::Counterclockwise, 5),
    ],
    [
        (3, Orientation::Counterclockwise, 1),
        (7, Orientation::Clockwise, 3),
        (1, Orientation::Clockwise, 5),
        (5, Orientation::Counterclockwise, 7),
    ],
    [
        (6, Orientation::Clockwise, 2),
        (2, Orientation::Counterclockwise, 3),
        (7, Orientation::Counterclockwise, 6),
        (3, Orientation::Clockwise, 7),
    ],
    [
        (4, Orientation::Clockwise, 0),
        (0, Orientation::Counterclockwise, 2),
        (6, Orientation::Counterclockwise, 4),
        (2, Orientation::Clockwise, 6),
    ],
    [
        (2, Orientation::Nothing, 0),
        (0, Orientation::Nothing, 1),
        (3, Orientation::Nothing, 2),
        (1, Orientation::Nothing, 3),
    ],
    [
        (7, Orientation::Nothing, 4),
        (6, Orientation::Nothing, 5),
        (5, Orientation::Nothing, 6),
        (4, Orientation::Nothing, 7),
    ],
    [
        (5, Orientation::Nothing, 0),
        (4, Orientation::Nothing, 1),
        (1, Orientation::Nothing, 4),
        (0, Orientation::Nothing, 5),
    ],
    [
        (7, Orientation::Nothing, 1),
        (5, Orientation::Nothing, 3),
        (3, Orientation::Nothing, 5),
        (1, Orientation::Nothing, 7),
    ],
    [
        (7, Orientation::Nothing, 2),
        (6, Orientation::Nothing, 3),
        (3, Orientation::Nothing, 6),
        (2, Orientation::Nothing, 7),
    ],
    [
        (6, Orientation::Nothing, 0),
        (4, Orientation::Nothing, 2),
        (2, Orientation::Nothing, 4),
        (0, Orientation::Nothing, 6),
    ],
    [
        (3, Orientation::Nothing, 0),
        (2, Orientation::Nothing, 1),
        (1, Orientation::Nothing, 2),
        (0, Orientation::Nothing, 3),
    ],
    [
        (6, Orientation::Nothing, 4),
        (4, Orientation::Nothing, 5),
        (7, Orientation::Nothing, 6),
        (5, Orientation::Nothing, 7),
    ],
    [
        (4, Orientation::Counterclockwise, 0),
        (0, Orientation::Clockwise, 1),
        (5, Orientation::Clockwise, 4),
        (1, Orientation::Counterclockwise, 5),
    ],
    [
        (5, Orientation::Counterclockwise, 1),
        (1, Orientation::Clockwise, 3),
        (7, Orientation::Clockwise, 5),
        (3, Orientation::Counterclockwise, 7),
    ],
    [
        (3, Orientation::Clockwise, 2),
        (7, Orientation::Counterclockwise, 3),
        (2, Orientation::Counterclockwise, 6),
        (6, Orientation::Clockwise, 7),
    ],
    [
        (2, Orientation::Clockwise, 0),
        (6, Orientation::Counterclockwise, 2),
        (0, Orientation::Counterclockwise, 4),
        (4, Orientation::Clockwise, 6),
    ],
    [
        (1, Orientation::Nothing, 0),
        (3, Orientation::Nothing, 1),
        (0, Orientation::Nothing, 2),
        (2, Orientation::Nothing, 3),
    ],
];

impl Cube<2> for Cube2 {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        let mut corners: [Corner; NUM_CORNERS] = Default::default();
        for (i, corner_index) in CORNER_INDICES.iter().enumerate() {
            corners[i] = Corner::new(*corner_index);
        }
        let mut color_map: [Color; NUM_FACES] = Default::default();
        for (face, color) in face_to_color {
            color_map[*face as usize] = *color;
        }
        Cube2 { corners, color_map }
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        let ref_cube = RefCube::<2>::parse_str(s)?;
        let mut corners: [Corner; NUM_CORNERS] = Default::default();
        for (colors, index) in CORNER_COLORS.iter().zip(CORNER_INDICES) {
            let (pos, orientation) = Cube2::find_corner(&ref_cube, colors)?;
            corners[pos as usize] = Corner { index, orientation };
        }
        Ok(Cube2 {
            corners,
            color_map: [
                Color::White,
                Color::Orange,
                Color::Green,
                Color::Red,
                Color::Blue,
                Color::Yellow,
            ],
        })
    }

    fn apply_turn(&mut self, turn: &Turn) {
        let turn_index: usize = turn.clone().into();
        let table: [(usize, Orientation, usize); 4] = TURN_TABLE[turn_index];
        for (index, orientation, _) in table {
            let corner: &mut Corner = &mut self.corners[index];
            corner.orientation = corner.orientation.compose(orientation);
        }
        let mut corners: [Corner; 4] = Default::default();
        for (i, (from_index, _, _)) in table.iter().enumerate() {
            corners[i] = self.corners[*from_index].clone();
        }
        for (i, (_, _, to_index)) in table.iter().enumerate() {
            self.corners[*to_index] = corners[i].clone();
        }
    }
}

impl From<Cube2> for RefCube<2> {
    fn from(cube2: Cube2) -> RefCube<2> {
        let mut ref_cube = RefCube::<2>::new();
        for (corner_index, corner) in cube2.corners.iter().enumerate() {
            let cube_faces = &CUBE_FACES[corner.index as usize];
            let mut colors = [Color::White; 3];
            for (i, face) in cube_faces.iter().enumerate() {
                colors[i] = cube2.color_map[*face as usize];
            }
            let colors = Cube2::apply_orientation(corner.orientation, &colors);

            let cube_faces = &CUBE_FACES[corner_index as usize];
            let coords = &COORDS[corner_index as usize];
            for ((face, coord), color) in
                cube_faces.iter().zip(coords).zip(colors)
            {
                *ref_cube.0[*face as usize].at_mut(coord) = color;
            }
        }
        ref_cube
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_turns(turns: &str) {
        let turns = Turns::parse_str(turns).unwrap();
        let mut cube2 = Cube2::new();
        cube2.apply_turns(&turns);
        let mut ref_cube = RefCube::<2>::new();
        ref_cube.apply_turns(&turns);
        assert_eq!(RefCube::<2>::from(cube2), ref_cube);
    }

    #[test]
    fn test_t_perm() {
        test_turns("RUR'U'R'FR2U'R'U'RUR'F'");
    }

    #[test]
    fn test_cube_in_a_cube() {
        test_turns("FLFU'RUF2L2U'L'BD'B'L2U");
    }

    #[test]
    fn turn_serialize() {
        for turn in Turn::all_turns::<2>().iter() {
            let turn_idx: usize = turn.clone().into();
            assert_eq!(*turn, turn_idx.into());
        }
    }
}
