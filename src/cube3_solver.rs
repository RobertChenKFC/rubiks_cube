use crate::color::{Color, ParsingErr};
use crate::cube::{Cube, NUM_FACES, RefCube};
use crate::cube2;
use crate::cube2::CornerIndex;
use crate::cube3;
use crate::cube3::{Cube3, EdgeIndex, CUBE3_SIZE, NUM_EDGES};
use crate::solver::{NaiveSolver, Solver};
use crate::turn::{CubeFace, Direction, Turn, Turns};

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::time::Instant;

const DBL_CORNER: [CornerIndex; 1] = [CornerIndex::DBL];
const DBL_EDGES: [EdgeIndex; 3] = [EdgeIndex::LB, EdgeIndex::BD, EdgeIndex::LD];

#[derive(Default, Debug, Clone, Eq)]
struct DblCube(Cube3);

impl Cube<3> for DblCube {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        DblCube(Cube3::from_colors(face_to_color))
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        Ok(DblCube(Cube3::parse_str(s)?))
    }

    fn apply_turn(&mut self, turn: &Turn) {
        self.0.apply_turn(turn);
    }
}

impl Hash for DblCube {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (pos, corner) in self.0.corners.corners.iter().enumerate() {
            if corner.index == CornerIndex::DBL {
                pos.hash(state);
                corner.orientation.hash(state);
                break;
            }
        }
        let mut positions: [usize; NUM_EDGES] = Default::default();
        for (pos, edge) in self.0.edges.edges.iter().enumerate() {
            positions[edge.index as usize] = pos;
        }
        for edge_index in DBL_EDGES.iter() {
            let pos = positions[*edge_index as usize];
            let edge = &self.0.edges.edges[pos];
            pos.hash(state);
            edge.orientation.hash(state);
        }
    }
}

impl PartialEq for DblCube {
    fn eq(&self, other: &Self) -> bool {
        for (corner1, corner2) in
            self.0.corners.corners.iter().zip(&other.0.corners.corners)
        {
            if corner1.index == CornerIndex::DBL
                && (corner1.index != corner2.index
                    || corner1.orientation != corner2.orientation)
            {
                return false;
            }
        }
        for (edge1, edge2) in
            self.0.edges.edges.iter().zip(&other.0.edges.edges)
        {
            if (edge1.index == EdgeIndex::LB
                || edge1.index == EdgeIndex::BD
                || edge1.index == EdgeIndex::LD)
                && (edge1.index != edge2.index
                    || edge1.orientation != edge2.orientation)
            {
                return false;
            }
        }
        true
    }
}

impl From<DblCube> for RefCube<3> {
    fn from(cube: DblCube) -> RefCube<3> {
        cube.0.into()
    }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone)]
pub struct CompactCube3(pub [u8; CUBE3_SIZE]);

impl Cube<3> for CompactCube3 {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        CompactCube3(Cube3::from_colors(face_to_color).serialize())
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        Ok(CompactCube3(Cube3::parse_str(s)?.serialize()))
    }

    fn apply_turn(&mut self, turn: &Turn) {
        let mut cube = Cube3::deserialize(&self.0).unwrap();
        cube.apply_turn(turn);
        self.0 = cube.serialize();
    }
}

impl From<CompactCube3> for RefCube<3> {
    fn from(cube: CompactCube3) -> RefCube<3> {
        Cube3::deserialize(&cube.0).unwrap().into()
    }
}

impl From<Cube3> for CompactCube3 {
    fn from(cube: Cube3) -> CompactCube3 {
        CompactCube3(cube.serialize())
    }
}

impl From<CompactCube3> for Cube3 {
    fn from(cube: CompactCube3) -> Cube3 {
        Cube3::deserialize(&cube.0).unwrap()
    }
}

pub struct Cube3Solver {
    all_states_to: HashMap<CompactCube3, Option<Turn>>
}

const NUL_TURN: u8 = 255;
const ENTRY_SIZE: usize = CUBE3_SIZE + 1;

impl Cube3Solver {
    pub fn new(table_path: &str) -> Cube3Solver {
        Cube3Solver {
            all_states_to: Cube3Solver::load_table(table_path)
        }
    }

    fn is_dbl_block_solved(cube: &Cube3) -> bool {
        let cube_corners = &cube.corners.corners;
        for corner_index in DBL_CORNER.iter() {
            let corner = &cube_corners[*corner_index as usize];
            if corner.index != *corner_index
                || corner.orientation != cube2::Orientation::Nothing
            {
                return false;
            }
        }
        let cube_edges = &cube.edges.edges;
        for edge_index in DBL_EDGES.iter() {
            let edge = &cube_edges[*edge_index as usize];
            if edge.index != *edge_index
                || edge.orientation != cube3::Orientation::Nothing
            {
                return false;
            }
        }
        true
    }

    // TODO: make this private
    pub fn all_ruf_turns() -> Turns {
        let mut all_turns = Vec::new();
        for face in [CubeFace::Right, CubeFace::Up, CubeFace::Front] {
            for dir in [
                Direction::Clockwise,
                Direction::Counterclockwise,
                Direction::Double,
            ] {
                all_turns.push(Turn {
                    face,
                    dir,
                    num_layers: 1,
                });
            }
        }
        Turns(all_turns)
    }

    pub fn gen_table(path: &str, len: usize) {
        let solved = CompactCube3::new();
        let table = NaiveSolver::get_all_states_within(&solved, len, &Cube3Solver::all_ruf_turns());
        let mut file = File::create(path).unwrap();
        for (i, (cube, turn)) in table.iter().enumerate() {
            if i % 1000000 == 0 {
                println!("Saved {} entries", i);
            }
            file.write_all(&cube.0).unwrap();
            let turn = match turn {
                Some(turn) => <Turn as Into<usize>>::into(turn.clone()) as u8,
                None => NUL_TURN,
            };
            file.write_all(&[turn]).unwrap();
        }
    }

    pub fn load_table(path: &str) -> HashMap<CompactCube3, Option<Turn>> {
        let now = Instant::now();
        let bytes = fs::read(path).unwrap();
        println!("Loading from file took {} us", now.elapsed().as_micros());

        let now = Instant::now();
        let num_entries = bytes.len() / ENTRY_SIZE;
        let mut table = HashMap::new();
        let mut cnt = 0;
        for i in (0..bytes.len()).step_by(ENTRY_SIZE) {
            if cnt % 1000000 == 0 {
                println!("Loaded {} entries", cnt);
            }
            let mut cube3: CompactCube3 = Default::default();
            cube3.0.copy_from_slice(&bytes[i..i + CUBE3_SIZE]);
            let turn = match bytes[i + CUBE3_SIZE] {
                NUL_TURN => None,
                turn => Some((turn as usize).into()),
            };
            table.insert(cube3, turn);
            cnt += 1;
        }
        println!("Converting into hash table took {} us", now.elapsed().as_micros());
        table
    }
}


impl Solver<3, Cube3> for Cube3Solver {
    fn solve(&self, cube: &Cube3) -> Turns {
        // Stage 1: solve the DBL 2x2 block
        let cur_dbl = DblCube(cube.clone());
        let solved_dbl = DblCube::new();
        // TODO: adjust the length. This seems to work for superflip, though.
        // However, the length is off by 1 for some reason, because the solution
        // only required 6 moves
        let mut turns1 =
            NaiveSolver::meet_in_the_middle(&cur_dbl, &solved_dbl, 7);
        println!("To DBL solved: {}", turns1);

        // Stage 2: solve the entire cube using only R, U, F
        let mut solved_dbl = cube.clone();
        solved_dbl.apply_turns(&turns1);
        let solved_dbl = CompactCube3(solved_dbl.serialize());
        // TODO: adjust the length. Currently, the table stores length 10 from
        // solved state, and the solver searchs length 9 from solved_dbl. This
        // seems to work for superflip
        let turns2 = NaiveSolver::find_turns_to_state(
            &solved_dbl,
            /*from_len=*/9,
            &self.all_states_to,
            &Cube3Solver::all_ruf_turns(),
        );
        turns1.0.extend(turns2.0);
        turns1
    }
}
