use crate::color::{Color, ParsingErr};
use crate::cube::{Cube, NUM_FACES, RefCube};
use crate::cube2;
use crate::cube2::{Cube2, Corner};
use crate::face::{Coord, Face};
use crate::turn::{CubeFace, Turn, Turns};

use std::collections::HashMap;
use std::hash::Hash;

#[derive(Default, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum EdgeIndex {
    #[default]
    UL,
    UF,
    UR,
    UB,
    LF,
    LB,
    LD,
    FR,
    FD,
    RB,
    RD,
    BD,
}

pub const NUM_EDGES: usize = 12;
const EDGE_INDICES: [EdgeIndex; NUM_EDGES] = [
    EdgeIndex::UL,
    EdgeIndex::UF,
    EdgeIndex::UR,
    EdgeIndex::UB,
    EdgeIndex::LF,
    EdgeIndex::LB,
    EdgeIndex::LD,
    EdgeIndex::FR,
    EdgeIndex::FD,
    EdgeIndex::RB,
    EdgeIndex::RD,
    EdgeIndex::BD,
];
const NUM_EDGE_FACES: usize = 2;
const EDGE_FACES: [[CubeFace; NUM_EDGE_FACES]; NUM_EDGES] = [
    [CubeFace::Up, CubeFace::Left],
    [CubeFace::Up, CubeFace::Front],
    [CubeFace::Up, CubeFace::Right],
    [CubeFace::Up, CubeFace::Back],
    [CubeFace::Left, CubeFace::Front],
    [CubeFace::Left, CubeFace::Back],
    [CubeFace::Left, CubeFace::Down],
    [CubeFace::Front, CubeFace::Right],
    [CubeFace::Front, CubeFace::Down],
    [CubeFace::Right, CubeFace::Back],
    [CubeFace::Right, CubeFace::Down],
    [CubeFace::Back, CubeFace::Down],
];
const COORDS: [[Coord; NUM_EDGE_FACES]; NUM_EDGES] = [
    [Coord::new(1, 0), Coord::new(0, 1)],
    [Coord::new(2, 1), Coord::new(0, 1)],
    [Coord::new(1, 2), Coord::new(0, 1)],
    [Coord::new(0, 1), Coord::new(0, 1)],
    [Coord::new(1, 2), Coord::new(1, 0)],
    [Coord::new(1, 0), Coord::new(1, 2)],
    [Coord::new(2, 1), Coord::new(1, 0)],
    [Coord::new(1, 2), Coord::new(1, 0)],
    [Coord::new(2, 1), Coord::new(0, 1)],
    [Coord::new(1, 2), Coord::new(1, 0)],
    [Coord::new(2, 1), Coord::new(1, 2)],
    [Coord::new(2, 1), Coord::new(2, 1)],
];

#[derive(Default, Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Orientation {
    #[default]
    Nothing,
    Flipped,
}

impl Orientation {
    fn compose(&self, other: Orientation) -> Orientation {
        match ((*self as usize) + (other as usize)) % 2 {
            0 => Orientation::Nothing,
            1 => Orientation::Flipped,
            _ => panic!("Not reachable"),
        }
    }
}

const NUM_ORIENTATIONS: usize = 2;
const ALL_ORIENTATIONS: [Orientation; NUM_ORIENTATIONS] =
    [Orientation::Nothing, Orientation::Flipped];

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Edge {
    pub index: EdgeIndex,
    pub orientation: Orientation,
}

impl Edge {
    fn new(index: EdgeIndex) -> Self {
        Edge {
            index,
            orientation: Orientation::Nothing,
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Cube3Edges {
    pub edges: [Edge; NUM_EDGES],
    color_map: [Color; NUM_FACES],
}

const NUM_TURNS: usize = 18;
const NUM_EDGES_PER_TURN: usize = 4;

impl Cube3Edges {
    fn apply_orientation(
        orientation: Orientation,
        colors: &[Color; NUM_EDGE_FACES],
    ) -> [Color; NUM_EDGE_FACES] {
        match orientation {
            Orientation::Nothing => colors.clone(),
            Orientation::Flipped => [colors[1], colors[0]],
        }
    }

    fn find_edge(
        colors: [Color; NUM_EDGE_FACES],
        ref_cube: &RefCube<3>,
    ) -> Result<(usize, Orientation), ParsingErr> {
        for ((pos, faces), coords) in
            EDGE_INDICES.iter().zip(EDGE_FACES).zip(COORDS)
        {
            let mut cube_colors: [Color; NUM_EDGE_FACES] = Default::default();
            for ((cube_color, face), coord) in
                cube_colors.iter_mut().zip(faces).zip(coords)
            {
                *cube_color = ref_cube.0[face as usize].at(&coord);
            }
            for orientation in ALL_ORIENTATIONS {
                let cube_colors =
                    Cube3Edges::apply_orientation(orientation, &cube_colors);
                if cube_colors == colors {
                    return Ok((*pos as usize, orientation));
                }
            }
        }
        Err(ParsingErr::InvalidColor)
    }

    fn apply_turn_ref(&mut self, turn: &Turn) {
        let mut ref_cube: RefCube<3> = self.clone().into();
        ref_cube.apply_turn(turn);
        *self = Cube3Edges::parse_str(&ref_cube.to_color_string()).unwrap();
    }

    fn apply_turns_ref(&mut self, turns: &Turns) {
        for turn in turns.iter() {
            self.apply_turn_ref(turn);
        }
    }

    pub fn gen_turn_table() {
        let mut table: [[(usize, Orientation, usize); NUM_EDGES_PER_TURN];
            NUM_TURNS] = Default::default();
        for turn in Turn::all_turns::<3>().iter() {
            let mut cube = Cube3Edges::new();
            cube.apply_turn_ref(turn);
            let mut orientation_table = HashMap::new();
            let mut permutation_table = HashMap::new();
            let mut indices = [0; NUM_EDGES_PER_TURN];
            let mut i = 0;
            for (index, edge) in EDGE_INDICES.iter().zip(cube.edges) {
                let from = edge.index as usize;
                let to = *index as usize;
                if from != to {
                    permutation_table.insert(from, to);
                    orientation_table.insert(from, edge.orientation);
                    indices[i] = from;
                    i += 1;
                }
            }
            let turn_idx: usize = turn.clone().into();
            let turn_entry = &mut table[turn_idx];
            for (from, turn_edge_entry) in indices.iter().zip(turn_entry) {
                let orientation = orientation_table[from];
                let to = permutation_table[from];
                *turn_edge_entry = (*from, orientation, to);
            }
        }
        println!("Table: {:?}", table);
    }
}

const TURN_TABLE: [[(usize, Orientation, usize); NUM_EDGES_PER_TURN];
    NUM_TURNS] = [
    [
        (1, Orientation::Nothing, 0),
        (2, Orientation::Nothing, 1),
        (3, Orientation::Nothing, 2),
        (0, Orientation::Nothing, 3),
    ],
    [
        (5, Orientation::Flipped, 0),
        (0, Orientation::Flipped, 4),
        (6, Orientation::Nothing, 5),
        (4, Orientation::Nothing, 6),
    ],
    [
        (4, Orientation::Nothing, 1),
        (8, Orientation::Flipped, 4),
        (1, Orientation::Flipped, 7),
        (7, Orientation::Nothing, 8),
    ],
    [
        (7, Orientation::Nothing, 2),
        (10, Orientation::Flipped, 7),
        (2, Orientation::Flipped, 9),
        (9, Orientation::Nothing, 10),
    ],
    [
        (9, Orientation::Nothing, 3),
        (3, Orientation::Nothing, 5),
        (11, Orientation::Flipped, 9),
        (5, Orientation::Flipped, 11),
    ],
    [
        (11, Orientation::Nothing, 6),
        (6, Orientation::Nothing, 8),
        (8, Orientation::Nothing, 10),
        (10, Orientation::Nothing, 11),
    ],
    [
        (2, Orientation::Nothing, 0),
        (3, Orientation::Nothing, 1),
        (0, Orientation::Nothing, 2),
        (1, Orientation::Nothing, 3),
    ],
    [
        (6, Orientation::Flipped, 0),
        (5, Orientation::Nothing, 4),
        (4, Orientation::Nothing, 5),
        (0, Orientation::Flipped, 6),
    ],
    [
        (8, Orientation::Flipped, 1),
        (7, Orientation::Flipped, 4),
        (4, Orientation::Flipped, 7),
        (1, Orientation::Flipped, 8),
    ],
    [
        (10, Orientation::Flipped, 2),
        (9, Orientation::Flipped, 7),
        (7, Orientation::Flipped, 9),
        (2, Orientation::Flipped, 10),
    ],
    [
        (11, Orientation::Flipped, 3),
        (9, Orientation::Nothing, 5),
        (5, Orientation::Nothing, 9),
        (3, Orientation::Flipped, 11),
    ],
    [
        (10, Orientation::Nothing, 6),
        (11, Orientation::Nothing, 8),
        (6, Orientation::Nothing, 10),
        (8, Orientation::Nothing, 11),
    ],
    [
        (3, Orientation::Nothing, 0),
        (0, Orientation::Nothing, 1),
        (1, Orientation::Nothing, 2),
        (2, Orientation::Nothing, 3),
    ],
    [
        (4, Orientation::Flipped, 0),
        (6, Orientation::Nothing, 4),
        (0, Orientation::Flipped, 5),
        (5, Orientation::Nothing, 6),
    ],
    [
        (7, Orientation::Flipped, 1),
        (1, Orientation::Nothing, 4),
        (8, Orientation::Nothing, 7),
        (4, Orientation::Flipped, 8),
    ],
    [
        (9, Orientation::Flipped, 2),
        (2, Orientation::Nothing, 7),
        (10, Orientation::Nothing, 9),
        (7, Orientation::Flipped, 10),
    ],
    [
        (5, Orientation::Nothing, 3),
        (11, Orientation::Flipped, 5),
        (3, Orientation::Nothing, 9),
        (9, Orientation::Flipped, 11),
    ],
    [
        (8, Orientation::Nothing, 6),
        (10, Orientation::Nothing, 8),
        (11, Orientation::Nothing, 10),
        (6, Orientation::Nothing, 11),
    ],
];

impl Cube<3> for Cube3Edges {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        let mut edges: [Edge; NUM_EDGES] = Default::default();
        for (edge, index) in edges.iter_mut().zip(EDGE_INDICES) {
            *edge = Edge::new(index);
        }
        let mut color_map: [Color; NUM_FACES] = Default::default();
        for (face, color) in face_to_color {
            color_map[*face as usize] = *color;
        }
        Cube3Edges { edges, color_map }
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        let ref_cube = RefCube::<3>::parse_str(s)?;
        let color_map = [
            Color::White,
            Color::Orange,
            Color::Green,
            Color::Red,
            Color::Blue,
            Color::Yellow,
        ];
        let mut edges: [Edge; NUM_EDGES] = Default::default();
        for (index, faces) in EDGE_INDICES.iter().zip(EDGE_FACES) {
            let mut colors: [Color; NUM_EDGE_FACES] = Default::default();
            for (color, face) in colors.iter_mut().zip(faces) {
                *color = color_map[face as usize];
            }
            let (pos, orientation) = Cube3Edges::find_edge(colors, &ref_cube)?;
            edges[pos] = Edge {
                index: *index,
                orientation,
            };
        }
        Ok(Cube3Edges { edges, color_map })
    }

    fn apply_turn(&mut self, turn: &Turn) {
        let turn_idx: usize = turn.clone().into();
        let mut edges: [Edge; NUM_EDGES_PER_TURN] = Default::default();
        let turn_entry = &TURN_TABLE[turn_idx];
        for (edge, (from, orientation, _)) in edges.iter_mut().zip(turn_entry) {
            *edge = self.edges[*from].clone();
            edge.orientation = edge.orientation.compose(*orientation);
        }
        for (edge, (_, _, to)) in edges.iter().zip(turn_entry) {
            self.edges[*to] = edge.clone();
        }
    }
}

impl From<Cube3Edges> for RefCube<3> {
    fn from(cube: Cube3Edges) -> RefCube<3> {
        let mut ref_cube = RefCube::<3>::new();
        // Fill all the color as white. We will fill in the edge colors later
        for face in ref_cube.0.iter_mut() {
            *face = Face::<3>::new(Color::White);
        }
        // Centers
        for (i, color) in cube.color_map.iter().enumerate() {
            *ref_cube.0[i].at_mut(&Coord::new(1, 1)) = *color;
        }
        // Edges
        for ((edge, faces), coords) in
            cube.edges.iter().zip(EDGE_FACES).zip(COORDS)
        {
            let Edge { index, orientation } = edge;
            let mut colors: [Color; NUM_EDGE_FACES] = Default::default();
            for (color, color_face) in
                colors.iter_mut().zip(EDGE_FACES[*index as usize])
            {
                *color = cube.color_map[color_face as usize];
            }
            let colors = Cube3Edges::apply_orientation(*orientation, &colors);
            for ((color, face), coord) in colors.iter().zip(faces).zip(coords) {
                *ref_cube.0[face as usize].at_mut(&coord) = *color;
            }
        }
        ref_cube
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Cube3 {
    pub edges: Cube3Edges,
    pub corners: Cube2,
}


#[derive(Debug)]
pub enum SerializationErr {
    InvalidEdgeIndex,
    InvalidEdgeOrientation,
    InvalidCornerIndex,
    InvalidCornerOrientation,
}

pub const CUBE3_SIZE: usize = 20;
impl Cube3 {
    pub fn serialize(&self) -> [u8; CUBE3_SIZE] {
        let mut i = 0;
        let mut bytes = [0; CUBE3_SIZE];
        for edge in &self.edges.edges {
            bytes[i] = ((edge.index as u8) << 4) | (edge.orientation as u8);
            i += 1;
        }
        for corner in &self.corners.corners {
            bytes[i] = ((corner.index as u8) << 4) | (corner.orientation as u8);
            i += 1;
        }
        bytes
    }

    pub fn deserialize(s: &[u8; CUBE3_SIZE]) -> Result<Cube3, SerializationErr> {
        let mut cube = Cube3::new();
        for (edge, byte) in cube.edges.edges.iter_mut().zip(s[0..NUM_EDGES].iter()) {
            let index = (byte >> 4) as usize;
            if index >= NUM_EDGES {
                return Err(SerializationErr::InvalidEdgeIndex);
            }
            let index = EDGE_INDICES[index];
            let orientation = (byte & 0xf) as usize;
            if orientation >= NUM_ORIENTATIONS {
                return Err(SerializationErr::InvalidEdgeOrientation);
            }
            let orientation = ALL_ORIENTATIONS[orientation];
            *edge = Edge { index, orientation };
        }
        for (corner, byte) in cube.corners.corners.iter_mut().zip(s[NUM_EDGES..CUBE3_SIZE].iter()) {
            let index = (byte >> 4) as usize;
            if index >= cube2::NUM_CORNERS {
                return Err(SerializationErr::InvalidCornerIndex);
            }
            let index = cube2::CORNER_INDICES[index];
            let orientation = (byte & 0xf) as usize;
            if orientation >= cube2::NUM_ORIENTATIONS {
                return Err(SerializationErr::InvalidCornerOrientation);
            }
            let orientation = cube2::ALL_ORIENTATIONS[orientation];
            *corner = Corner { index, orientation };
        }
        Ok(cube)
    }
}

impl Cube<3> for Cube3 {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        Cube3 {
            edges: Cube3Edges::from_colors(face_to_color),
            corners: Cube2::from_colors(face_to_color),
        }
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        let edges = Cube3Edges::parse_str(s)?;
        let ref_cube3 = RefCube::<3>::parse_str(s)?;
        let mut ref_cube2 = RefCube::<2>::new();
        for face in 0..NUM_FACES {
            for row in 0..2 {
                for col in 0..2 {
                    *ref_cube2.0[face].at_mut(&Coord::new(row, col)) =
                        ref_cube3.0[face].at(&Coord::new(row * 2, col * 2));
                }
            }
        }
        let corners = Cube2::parse_str(&ref_cube2.to_color_string())?;
        Ok(Cube3 { edges, corners })
    }

    fn apply_turn(&mut self, turn: &Turn) {
        self.edges.apply_turn(turn);
        self.corners.apply_turn(turn);
    }
}

impl From<Cube3> for RefCube<3> {
    fn from(cube: Cube3) -> RefCube<3> {
        let mut ref_cube3: RefCube<3> = cube.edges.into();
        let ref_cube2: RefCube<2> = cube.corners.into();
        for face in 0..NUM_FACES {
            for row in 0..2 {
                for col in 0..2 {
                    *ref_cube3.0[face].at_mut(&Coord::new(row * 2, col * 2)) =
                        ref_cube2.0[face].at(&Coord::new(row, col));
                }
            }
        }
        ref_cube3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_turns<C: Cube<3>>(turns: &str) {
        let turns = Turns::parse_str(turns).unwrap();
        let mut cube3 = C::new();
        let mut ref_cube: RefCube<3> = cube3.clone().into();
        cube3.apply_turns(&turns);
        ref_cube.apply_turns(&turns);
        let cube3: RefCube<3> = cube3.into();
        assert_eq!(cube3, ref_cube);
    }

    #[test]
    fn test_t_perm_edges() {
        test_turns::<Cube3Edges>("RUR'U'R'FR2U'R'U'RUR'F'");
    }

    #[test]
    fn test_cube_in_a_cube_edges() {
        test_turns::<Cube3Edges>("FLFU'RUF2L2U'L'BD'B'L2U");
    }

    #[test]
    fn test_t_perm() {
        test_turns::<Cube3>("RUR'U'R'FR2U'R'U'RUR'F'");
    }

    #[test]
    fn test_y_perm() {
        test_turns::<Cube3>("FRU'R'U'RUR'F'RUR'U'R'FRF'");
    }

    #[test]
    fn test_g_perm() {
        test_turns::<Cube3>("RUR'U'DR2U'RU'R'UR'UR2UD'");
    }

    #[test]
    fn test_cube_in_a_cube() {
        test_turns::<Cube3>(
            "FLFU'RUF2L2U'L'BD'B'L2UB2R'DRD'R'DRUR'D'RDR'D'RU'B2",
        );
    }

    #[test]
    fn test_superflip() {
        test_turns::<Cube3>("UR2FBRB2RU2LB2RU'D'R2FR'LB2U2F2");
    }

    #[test]
    fn test_serialization() {
        let mut cube = Cube3::new();
        let turns = Turns::parse_str("UR2FBRB2RU2LB2RU'D'R2FR'LB2U2F2").unwrap();
        cube.apply_turns(&turns);
        let cube2 = Cube3::deserialize(&cube.serialize()).unwrap();
        assert_eq!(cube, cube2);
    }
}
