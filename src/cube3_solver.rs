use crate::color::{Color, ParsingErr};
use crate::cube::{Cube, DisplayCube, NUM_FACES, RefCube};
use crate::cube2;
use crate::cube2::{Corner, CornerIndex, NUM_CORNERS, CORNER_INDICES};
use crate::cube3;
use crate::cube3::{CUBE3_SIZE, Cube3, Edge, EdgeIndex, NUM_EDGES, EDGE_INDICES};
use crate::solver::{CubeTable, NaiveSolver, Solver};
use crate::turn::{CubeFace, Direction, Turn, Turns};

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};
use std::io;
use std::io::Write;
use std::marker::PhantomData;
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

pub trait SerializableCube3<const N: usize>:
    Cube<3> + From<Cube3> + Ord
{
    fn to_bytes(&self) -> &[u8];
    fn from_bytes(bytes: [u8; N]) -> Self;
}

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
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

impl SerializableCube3<CUBE3_SIZE> for CompactCube3 {
    fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    fn from_bytes(bytes: [u8; CUBE3_SIZE]) -> Self {
        CompactCube3(bytes)
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

#[derive(Default, Clone, Copy)]
pub struct CubeHasher(u64);

impl Hasher for CubeHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        if bytes.len() == 10 {
            let mut buf = [0u8; 16];
            let len = 16.min(bytes.len());
            buf[..len].copy_from_slice(&bytes[..len]);
            self.0 ^= (u128::from_le_bytes(buf) % 18446744073709551557) as u64;
        }
    }
}

impl BuildHasher for CubeHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        *self
    }
}

const RUF_CUBE3_SIZE: usize = 10;

#[derive(Default, Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct RufCube3 {
    bytes: [u8; RUF_CUBE3_SIZE],
}

impl SerializableCube3<RUF_CUBE3_SIZE> for RufCube3 {
    fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn from_bytes(bytes: [u8; RUF_CUBE3_SIZE]) -> Self {
        RufCube3 { bytes }
    }
}

const NUM_RUF_EDGES: usize = 9;
const RUF_EDGES: [EdgeIndex; NUM_RUF_EDGES] = [
    EdgeIndex::UL,
    EdgeIndex::UF,
    EdgeIndex::UR,
    EdgeIndex::UB,
    EdgeIndex::LF,
    EdgeIndex::FR,
    EdgeIndex::FD,
    EdgeIndex::RB,
    EdgeIndex::RD,
];
const RUF_EDGE_INDEX_OFFSET: usize = 0;
// 4 bits are needed to encode 9 edges
const RUF_EDGE_INDEX_SIZE: usize = 4;
const RUF_EDGE_INDEX_MASK: u64 = 0b1111;
const RUF_EDGE_ORIENTATION_OFFSET: usize = RUF_EDGE_INDEX_SIZE * NUM_RUF_EDGES;
// 1 bit is needed to encode an edge orientation
const RUF_EDGE_ORIENTATION_SIZE: usize = 1;
const RUF_EDGE_ORIENTATION_MASK: u64 = 0b1;
const NUM_RUF_CORNERS: usize = 7;
const RUF_CORNERS: [CornerIndex; NUM_RUF_CORNERS] = [
    CornerIndex::DLF,
    CornerIndex::DRB,
    CornerIndex::DFR,
    CornerIndex::ULB,
    CornerIndex::UFL,
    CornerIndex::UBR,
    CornerIndex::URF,
];
const RUF_CORNER_INDEX_OFFSET: usize =
    RUF_EDGE_ORIENTATION_OFFSET + RUF_EDGE_ORIENTATION_SIZE * NUM_RUF_EDGES;
// 3 bits are needed to encode 7 corners
const RUF_CORNER_INDEX_SIZE: usize = 3;
const RUF_CORNER_INDEX_MASK: u64 = 0b111;
const RUF_CORNER_ORIENTATION_OFFSET: usize =
    RUF_CORNER_INDEX_OFFSET + RUF_CORNER_INDEX_SIZE * NUM_RUF_CORNERS;
// 2 bits are needed to encode a corner orientation
const RUF_CORNER_ORIENTATION_SIZE: usize = 2;
const RUF_CORNER_ORIENTATION_MASK: u64 = 0b11;

impl RufCube3 {
    fn set_bits(&mut self, bits: u8, size: usize, offset: usize) {
        let start_index = offset / 8;
        let start_offset = offset % 8;
        let end = offset + size - 1;
        let end_index = end / 8;
        self.bytes[start_index] |= bits << start_offset;
        if start_index != end_index {
            self.bytes[end_index] |= bits >> (8 - start_offset);
        }
    }

    fn get_bits(&self, size: usize, offset: usize) -> u8 {
        let start_index = offset / 8;
        let start_offset = offset % 8;
        let end = offset + size - 1;
        let end_index = end / 8;
        let mut bits = self.bytes[start_index] >> start_offset;
        if start_index != end_index {
            bits |= self.bytes[end_index] << (8 - start_offset);
        }
        bits & ((1 << size) - 1)
    }

    pub fn apply_turn_ref(&mut self, turn: &Turn) {
        let mut cube3: Cube3 = self.clone().into();
        cube3.apply_turn(turn);
        *self = cube3.into();
    }

    fn find_pos<T: Clone + Into<usize>>(table: &[T], elem: usize) -> usize {
        for (pos, val) in table.iter().enumerate() {
            let val: usize = val.clone().into();
            if val == elem {
                return pos;
            }
        }
        panic!("Position not found");
    }

    fn find_cycle<T: Clone + Into<usize>>(permutation: &[usize], table: &[T]) -> [usize; 4]{
        let mut cycle = [0; 4];
        let mut cur = None;
        for (pos, val) in permutation.iter().enumerate() {
            if pos != *val {
                cur = Some(pos);
                cycle[0] = *val;
                break;
            }
        }
        for i in 1..4 {
            for (pos, val) in permutation.iter().enumerate() {
                if cur == Some(*val) {
                    cur = Some(pos);
                    cycle[i] = *val;
                    break;
                }
            }
        }
        for val in &mut cycle {
            *val = RufCube3::find_pos(table, *val);
        }
        cycle
    }

    pub fn gen_table() {
        for turn in Cube3Solver::<_, RufCube3>::all_ruf_turns().iter() {
            let mut cube = Cube3::new();
            cube.apply_turn(turn);

            let mut table = [0; 12];
            let mut orientation = [0; 12];
            for (pos, edge) in cube.edges.edges.iter().enumerate() {
                table[pos] = edge.index as usize;
                orientation[edge.index as usize] = edge.orientation as usize;
            }
            let cycle = RufCube3::find_cycle(&table, &RUF_EDGES);
            let orientation: Vec<_> = cycle.iter().map(|index| orientation[RUF_EDGES[*index] as usize]).collect();
            println!("Turn: {}, Edge Cycle: {:?}", turn, cycle);
            println!("Turn: {}, Edge Orientation: {:?}", turn, orientation);

            let mut table = [0; 8];
            let mut orientation = [0; 8];
            for (pos, corner) in cube.corners.corners.iter().enumerate() {
                table[pos] = corner.index as usize;
                orientation[corner.index as usize] = corner.orientation as usize;
            }
            let cycle = RufCube3::find_cycle(&table, &RUF_CORNERS);
            let orientation: Vec<_> = cycle.iter().map(|index| orientation[RUF_CORNERS[*index] as usize]).collect();
            println!("Turn: {}, Corner Cycle: {:?}", turn, cycle);
            println!("Turn: {}, Corner Orientation: {:?}", turn, orientation);
        }
    }

    fn orient<const MODULO: u8>(arr: &mut [u8], permutation: &[usize], orientation: &[u8]) {
        for (pos, cur_orientation) in permutation.iter().zip(orientation) {
            arr[*pos] = (arr[*pos] + cur_orientation) % MODULO;
        }
    }

    fn permute(arr: &mut [u8], permutation: &[usize], dir: Direction) {
        let a = permutation[0];
        let b = permutation[1];
        let c = permutation[2];
        let d = permutation[3];
        match dir {
            Direction::Clockwise => {
                let t = arr[d];
                arr[d] = arr[c];
                arr[c] = arr[b];
                arr[b] = arr[a];
                arr[a] = t;
            },
            Direction::Counterclockwise => {
                let t = arr[a];
                arr[a] = arr[b];
                arr[b] = arr[c];
                arr[c] = arr[d];
                arr[d] = t;
            },
            Direction::Double => {
                let t = arr[a];
                arr[a] = arr[c];
                arr[c] = t;
                let t = arr[b];
                arr[b] = arr[d];
                arr[d] = t;
            },
        }
    }
}

impl Cube<3> for RufCube3 {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        Cube3::from_colors(face_to_color).into()
    }

    fn parse_str(s: &str) -> Result<Self, ParsingErr> {
        let cube3 = Cube3::parse_str(s)?;
        Ok(cube3.into())
    }

    fn apply_turn(&mut self, turn: &Turn) {
        let mut buf = [0u8; 2];
        buf.copy_from_slice(&self.bytes[0..2]);
        let first_16_bits = u16::from_le_bytes(buf) as u64;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.bytes[2..10]);
        let last_64_bits = u64::from_le_bytes(buf);

        let mut edge_permutation = [0u8; NUM_RUF_EDGES];
        let mut offset = 0;
        for edge in &mut edge_permutation[0..4] {
            *edge = ((first_16_bits >> offset) & RUF_EDGE_INDEX_MASK) as u8;
            offset += RUF_EDGE_INDEX_SIZE;
        }
        let mut offset = 0;
        for edge in &mut edge_permutation[4..9] {
            *edge = ((last_64_bits >> offset) & RUF_EDGE_INDEX_MASK) as u8;
            offset += RUF_EDGE_INDEX_SIZE;
        }
        // TODO: use the bitwise trick instead of storing them as u8 arrays
        let mut edge_orientation = [0u8; NUM_RUF_EDGES];
        for edge in &mut edge_orientation {
            *edge = ((last_64_bits >> offset) & RUF_EDGE_ORIENTATION_MASK) as u8;
            offset += RUF_EDGE_ORIENTATION_SIZE;
        }

        let mut corner_permutation = [0u8; NUM_RUF_CORNERS];
        for corner in &mut corner_permutation {
            *corner = ((last_64_bits >> offset) & RUF_CORNER_INDEX_MASK) as u8;
            offset += RUF_CORNER_INDEX_SIZE;
        }
        let mut corner_orientation = [0u8; NUM_RUF_CORNERS];
        for corner in &mut corner_orientation {
            *corner = ((last_64_bits >> offset) & RUF_CORNER_ORIENTATION_MASK) as u8;
            offset += RUF_CORNER_ORIENTATION_SIZE;
        }

        let edge_permutation_table = match turn.face {
            CubeFace::Right => [5, 2, 7, 8],
            CubeFace::Up => [1, 0, 3, 2],
            CubeFace::Front => [4, 1, 5, 6],
            _ => panic!("Invalid face for RUF cubes"),
        };
        let corner_permutation_table = match turn.face {
            CubeFace::Right => [5, 1, 2, 6],
            CubeFace::Up => [4, 3, 5, 6],
            CubeFace::Front => [0, 4, 6, 2],
            _ => panic!("Invalid face for RUF cubes"),
        };
        if turn.face != CubeFace::Up {
            let edge_orientation_table = match turn.dir {
                Direction::Clockwise => [0, 1, 0, 1],
                Direction::Counterclockwise => [1, 0, 1, 0],
                Direction::Double => [1, 1, 1, 1],
            };
            let corner_orientation_table = match turn.dir {
                Direction::Clockwise => [1, 2, 1, 2],
                Direction::Counterclockwise => [1, 2, 1, 2],
                Direction::Double => [0, 0, 0, 0]
            };
            RufCube3::orient::<2>(&mut edge_orientation, &edge_permutation_table, &edge_orientation_table);
            RufCube3::orient::<3>(&mut corner_orientation, &corner_permutation_table, &corner_orientation_table);
        }

        RufCube3::permute(&mut edge_permutation, &edge_permutation_table, turn.dir);
        RufCube3::permute(&mut edge_orientation, &edge_permutation_table, turn.dir);
        RufCube3::permute(&mut corner_permutation, &corner_permutation_table, turn.dir);
        RufCube3::permute(&mut corner_orientation, &corner_permutation_table, turn.dir);

        let mut offset = 0;
        let mut first_16_bits = 0u16;
        for edge in &edge_permutation[0..4] {
            first_16_bits |= (*edge as u16) << offset;
            offset += RUF_EDGE_INDEX_SIZE;
        }
        let mut offset = 0;
        let mut last_64_bits = 0u64;
        for edge in &edge_permutation[4..9] {
            last_64_bits |= (*edge as u64) << offset;
            offset += RUF_EDGE_INDEX_SIZE;
        }
        for edge in edge_orientation {
            last_64_bits |= (edge as u64) << offset;
            offset += RUF_EDGE_ORIENTATION_SIZE;
        }
        for corner in corner_permutation {
            last_64_bits |= (corner as u64) << offset;
            offset += RUF_CORNER_INDEX_SIZE;
        }
        for corner in corner_orientation {
            last_64_bits |= (corner as u64) << offset;
            offset += RUF_CORNER_ORIENTATION_SIZE;
        }

        self.bytes[0..2].copy_from_slice(&first_16_bits.to_le_bytes());
        self.bytes[2..10].copy_from_slice(&last_64_bits.to_le_bytes());
    }
}

impl From<Cube3> for RufCube3 {
    fn from(cube3: Cube3) -> RufCube3 {
        let mut cube: RufCube3 = Default::default();

        let edges = &cube3.edges.edges;
        for (i, pos) in RUF_EDGES.iter().enumerate() {
            let edge = &edges[*pos as usize];
            cube.set_bits(
                edge.index as u8,
                RUF_EDGE_INDEX_SIZE,
                i * RUF_EDGE_INDEX_SIZE + RUF_EDGE_INDEX_OFFSET,
            );
            cube.set_bits(
                edge.orientation as u8,
                RUF_EDGE_ORIENTATION_SIZE,
                i * RUF_EDGE_ORIENTATION_SIZE + RUF_EDGE_ORIENTATION_OFFSET,
            );
        }

        let corners = &cube3.corners.corners;
        for (i, pos) in RUF_CORNERS.iter().enumerate() {
            let corner = &corners[*pos as usize];
            cube.set_bits(
                corner.index as u8,
                RUF_CORNER_INDEX_SIZE,
                i * RUF_CORNER_INDEX_SIZE + RUF_CORNER_INDEX_OFFSET,
            );
            cube.set_bits(
                corner.orientation as u8,
                RUF_CORNER_ORIENTATION_SIZE,
                i * RUF_CORNER_ORIENTATION_SIZE + RUF_CORNER_ORIENTATION_OFFSET,
            );
        }

        cube
    }
}

impl From<RufCube3> for Cube3 {
    fn from(cube: RufCube3) -> Cube3 {
        let mut cube3 = Cube3::new();

        let edges = &mut cube3.edges.edges;
        for (i, pos) in RUF_EDGES.iter().enumerate() {
            let index = cube.get_bits(
                RUF_EDGE_INDEX_SIZE,
                i * RUF_EDGE_INDEX_SIZE + RUF_EDGE_INDEX_OFFSET,
            );
            let orientation = cube.get_bits(
                RUF_EDGE_ORIENTATION_SIZE,
                i * RUF_EDGE_ORIENTATION_SIZE + RUF_EDGE_ORIENTATION_OFFSET,
            );
            let orientation = cube3::ALL_ORIENTATIONS[orientation as usize];
            edges[*pos as usize] = Edge {
                index: EDGE_INDICES[index as usize],
                orientation,
            };
        }

        let corners = &mut cube3.corners.corners;
        for (i, pos) in RUF_CORNERS.iter().enumerate() {
            let index = cube.get_bits(
                RUF_CORNER_INDEX_SIZE,
                i * RUF_CORNER_INDEX_SIZE + RUF_CORNER_INDEX_OFFSET,
            );
            let orientation = cube.get_bits(
                RUF_CORNER_ORIENTATION_SIZE,
                i * RUF_CORNER_ORIENTATION_SIZE + RUF_CORNER_ORIENTATION_OFFSET,
            );
            let orientation = cube2::ALL_ORIENTATIONS[orientation as usize];
            corners[*pos as usize] = Corner {
                index: CORNER_INDICES[index as usize],
                orientation,
            };
        }

        cube3
    }
}

impl From<RufCube3> for RefCube<3> {
    fn from(cube: RufCube3) -> RefCube<3> {
        let cube3: Cube3 = cube.into();
        cube3.into()
    }
}

struct CubeVecTable<'a, const N: usize, C: SerializableCube3<N>> {
    vector: &'a Vec<(C, Option<Turn>)>,
}

impl<'a, const N: usize, C: SerializableCube3<N>> CubeVecTable<'a, N, C> {
    fn new(vector: &'a Vec<(C, Option<Turn>)>) -> Self {
        Self { vector }
    }
}

impl<'a, const N: usize, C: SerializableCube3<N>> CubeTable<3, C>
    for CubeVecTable<'a, N, C>
{
    fn find(&self, cube: &C) -> Option<Turn> {
        for (candidate, turn) in self.vector {
            if cube == candidate {
                return turn.clone();
            }
        }
        panic!("Cube not found in vector");
    }
}

pub struct Cube3Solver<
    const N: usize,
    C: SerializableCube3<N>,
    S: BuildHasher + Clone = RandomState,
> {
    all_states_to: Vec<(C, Option<Turn>)>,
    // TODO: remove this (and the type parameter)
    hasher_type: PhantomData<S>,
}

pub struct CubeTableIter<const N: usize, C: SerializableCube3<N>> {
    i: usize,
    bytes: Vec<u8>,
    cube_type: PhantomData<C>,
}

impl<const N: usize, C: SerializableCube3<N>> CubeTableIter<N, C> {
    const ENTRY_SIZE: usize = N + 1;

    fn new(path: &str) -> Self {
        let now = Instant::now();
        let bytes = fs::read(path).unwrap();
        println!("Loading from file took {} us", now.elapsed().as_micros());
        CubeTableIter {
            i: 0,
            bytes,
            cube_type: PhantomData,
        }
    }
}

impl<const N: usize, C: SerializableCube3<N>> Iterator for CubeTableIter<N, C> {
    type Item = (C, Option<Turn>);

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.i * Self::ENTRY_SIZE;
        if offset >= self.bytes.len() {
            return None;
        }
        let cube3 =
            C::from_bytes(self.bytes[offset..offset + N].try_into().unwrap());
        let turn = match self.bytes[offset + N] {
            NUL_TURN => None,
            turn => Some((turn as usize).into()),
        };
        self.i += 1;
        if self.i % 1000000 == 0 {
            println!("Loaded {} entries", self.i);
        }
        Some((cube3, turn))
    }
}

const NUL_TURN: u8 = 255;

impl<const N: usize, C: SerializableCube3<N>, S: BuildHasher + Clone + Default>
    Cube3Solver<N, C, S>
{
    const ENTRY_SIZE: usize = N + 1;

    pub fn new(table_path: &str) -> Cube3Solver<N, C, S> {
        Cube3Solver {
            all_states_to: Cube3Solver::<N, C, S>::load_table(table_path),
            hasher_type: PhantomData,
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

    fn all_ruf_turns() -> Turns {
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
        let solved = C::new();
        let solver: NaiveSolver<_, _, S> = NaiveSolver::new();
        let table = solver.get_all_states_within(
            &solved,
            len,
            &Cube3Solver::<N, C>::all_ruf_turns(),
        );

        let mut flat_table = Vec::new();
        flat_table.reserve(table.len());
        for (cube, turn) in table {
            flat_table.push((cube.clone(), turn.clone()));
        }
        flat_table.sort_unstable_by(|(cube1, _), (cube2, _)| cube1.cmp(cube2));

        let mut file = File::create(path).unwrap();
        for (i, (cube, turn)) in flat_table.iter().enumerate() {
            if i % 1000000 == 0 {
                println!("Saved {} entries", i);
            }
            file.write_all(&cube.to_bytes()).unwrap();
            let turn = match turn {
                Some(turn) => <Turn as Into<usize>>::into(turn.clone()) as u8,
                None => NUL_TURN,
            };
            file.write_all(&[turn]).unwrap();
        }
    }

    pub fn load_table_elems(path: &str) -> CubeTableIter<N, C> {
        CubeTableIter::new(path)
    }

    pub fn load_table(path: &str) -> Vec<(C, Option<Turn>)> {
        let table_elems = Cube3Solver::<N, C, S>::load_table_elems(path);
        let now = Instant::now();
        let mut table = Vec::new();
        for elem in table_elems {
            table.push(elem);
        }
        println!(
            "Converting into hash table took {} us",
            now.elapsed().as_micros()
        );
        table
    }
}

impl<const N: usize, C: SerializableCube3<N>, S: BuildHasher + Clone + Default>
    Solver<3, Cube3> for Cube3Solver<N, C, S>
{
    fn solve(&self, cube: &Cube3) -> Turns {
        // Stage 1: solve the DBL 2x2 block
        let cur_dbl = DblCube(cube.clone());
        let solved_dbl = DblCube::new();
        let naive_solver: NaiveSolver<_, _> = NaiveSolver::new();
        // TODO: adjust the length. This seems to work for superflip, though.
        // However, the length is off by 1 for some reason, because the solution
        // only required 6 moves
        let mut turns =
            naive_solver.meet_in_the_middle(&cur_dbl, &solved_dbl, 7);
        println!("To DBL solved: {}", turns);

        // Stage 2: solve the entire cube using only R, U, F
        let mut solved_dbl = cube.clone();
        solved_dbl.apply_turns(&turns);
        let solved_dbl: C = solved_dbl.into();
        let naive_solver: NaiveSolver<_, _, S> = NaiveSolver::new();
        let mut from_solved_dbl: Vec<_> = naive_solver
            .get_all_states_within(
                &solved_dbl,
                /*from_len=*/ 9,
                &Cube3Solver::<N, C>::all_ruf_turns(),
            )
            .into_iter()
            .collect();
        from_solved_dbl
            .sort_unstable_by(|(cube1, _), (cube2, _)| cube1.cmp(cube2));
        // Both `from_solved_dbl` and `self.all_states_to` are sorted, we just
        // have to find a cube state that is in both. We can do this linearly
        // similar to how merge sort is done.
        let mut i = 0;
        let mut j = 0;
        let mut found = false;
        while i < from_solved_dbl.len() && j < self.all_states_to.len() {
            let cube1 = &from_solved_dbl[i].0;
            let cube2 = &self.all_states_to[j].0;
            match cube1.cmp(cube2) {
                Ordering::Less => i += 1,
                Ordering::Equal => {
                    found = true;
                    break;
                }
                Ordering::Greater => j += 1,
            }
        }
        assert!(found);
        let turns_from = NaiveSolver::<_, C, S>::get_turns_to_state(
            &from_solved_dbl[i].0,
            &CubeVecTable::new(&from_solved_dbl),
        )
        .0;
        let turns_to = NaiveSolver::<_, C, S>::get_turns_from_state(
            &self.all_states_to[j].0,
            &CubeVecTable::new(&self.all_states_to),
        )
        .0;
        turns.0.extend(turns_from);
        turns.0.extend(turns_to);
        turns
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
    fn test_ruf_t_perm() {
        test_turns::<RufCube3>("RUR'U'R'FR2U'R'U'RUR'F'");
    }
}
