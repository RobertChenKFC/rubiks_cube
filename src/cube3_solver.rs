use crate::color::{Color, ParsingErr};
use crate::cube::{Cube, NUM_FACES, RefCube};
use crate::cube2;
use crate::cube2::CornerIndex;
use crate::cube3;
use crate::cube3::{Cube3, EdgeIndex, NUM_EDGES};
use crate::solver::{NaiveSolver, Solver};
use crate::turn::{CubeFace, Direction, Turn, Turns};

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

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

pub struct Cube3Solver;

impl Cube3Solver {
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
}

impl Solver<3, Cube3> for Cube3Solver {
    fn solve(cube: &Cube3) -> Turns {
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
        let solved = Cube3::new();
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
        let all_turns = Turns(all_turns);
        // TODO: adjust the length
        let turns2 = NaiveSolver::meet_in_the_middle_with_turns(
            &solved_dbl,
            &solved,
            19,
            &all_turns,
        );
        turns1.0.extend(turns2.0);
        turns1
    }
}
