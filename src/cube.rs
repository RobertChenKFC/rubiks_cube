use crate::color::{Color, ParsingErr};
use crate::face::{Coord, Face};
use crate::turn::{CubeFace, Direction, Turn, Turns};

use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash;

const NUM_FACES: usize = 6;
pub trait Cube<const N: usize>:
    Sized + Clone + hash::Hash + PartialEq + Eq + fmt::Debug + Into<RefCube<N>>
{
    fn new() -> Self {
        Cube::<N>::from_colors(&[
            (CubeFace::Up, Color::White),
            (CubeFace::Left, Color::Orange),
            (CubeFace::Front, Color::Green),
            (CubeFace::Right, Color::Red),
            (CubeFace::Back, Color::Blue),
            (CubeFace::Down, Color::Yellow),
        ])
    }

    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self;

    fn parse_str(s: &str) -> Result<Self, ParsingErr>;

    fn apply_turn(&mut self, turn: &Turn);

    fn apply_turns(&mut self, turns: &Turns) {
        for turn in turns.iter() {
            self.apply_turn(turn);
        }
    }

    fn from_corner_colors(&self) -> Self {
        let ref_cube: RefCube<N> = self.clone().into();
        let back_color = ref_cube.0[CubeFace::Back as usize].at(&Coord {
            row: N - 1,
            col: N - 1,
        });
        let left_color = ref_cube.0[CubeFace::Left as usize]
            .at(&Coord { row: N - 1, col: 0 });
        let down_color = ref_cube.0[CubeFace::Down as usize]
            .at(&Coord { row: N - 1, col: 0 });
        let front_color = back_color.opposite();
        let right_color = left_color.opposite();
        let up_color = down_color.opposite();
        Cube::from_colors(&[
            (CubeFace::Up, up_color),
            (CubeFace::Left, left_color),
            (CubeFace::Front, front_color),
            (CubeFace::Right, right_color),
            (CubeFace::Back, back_color),
            (CubeFace::Down, down_color),
        ])
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct RefCube<const N: usize>([Face<N>; NUM_FACES]);

struct SideInfo {
    face: CubeFace,
    fixed_idx: usize,
    fixed_idx_is_row: bool,
    flip_varying_idx: bool,
}

impl<const N: usize> RefCube<N> {
    fn default() -> Self {
        RefCube([
            Face::new(Color::White),
            Face::new(Color::White),
            Face::new(Color::White),
            Face::new(Color::White),
            Face::new(Color::White),
            Face::new(Color::White),
        ])
    }

    fn display_empty_row(&self, f: &mut Formatter) -> fmt::Result {
        for _ in 0..N {
            write!(f, "  ")?;
        }
        Ok(())
    }

    fn display_faces(
        &self,
        f: &mut Formatter,
        faces: &[Option<CubeFace>],
    ) -> fmt::Result {
        for i in 0..N {
            for face in faces {
                match face {
                    Some(face) => self.0[*face as usize].display_row(f, i)?,
                    None => self.display_empty_row(f)?,
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }

    fn build_coord(mut varying_idx: usize, side_info: &SideInfo) -> Coord {
        let SideInfo {
            face: _,
            fixed_idx,
            fixed_idx_is_row,
            flip_varying_idx,
        } = side_info;
        if *flip_varying_idx {
            varying_idx = N - 1 - varying_idx;
        }
        if *fixed_idx_is_row {
            Coord {
                row: *fixed_idx,
                col: varying_idx,
            }
        } else {
            Coord {
                row: varying_idx,
                col: *fixed_idx,
            }
        }
    }

    fn cycle_sides(&mut self, turn: &Turn) {
        let side_infos = match turn.face {
            CubeFace::Up => [
                SideInfo {
                    face: CubeFace::Front,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Left,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Back,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Right,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
            ],
            CubeFace::Left => [
                SideInfo {
                    face: CubeFace::Front,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Down,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Back,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: true,
                },
                SideInfo {
                    face: CubeFace::Up,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
            ],
            CubeFace::Front => [
                SideInfo {
                    face: CubeFace::Up,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Right,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Down,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: true,
                },
                SideInfo {
                    face: CubeFace::Left,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: true,
                },
            ],
            CubeFace::Right => [
                SideInfo {
                    face: CubeFace::Front,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Up,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Back,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: true,
                },
                SideInfo {
                    face: CubeFace::Down,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
            ],
            CubeFace::Back => [
                SideInfo {
                    face: CubeFace::Up,
                    fixed_idx: 0,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Left,
                    fixed_idx: 0,
                    fixed_idx_is_row: false,
                    flip_varying_idx: true,
                },
                SideInfo {
                    face: CubeFace::Down,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: true,
                },
                SideInfo {
                    face: CubeFace::Right,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: false,
                    flip_varying_idx: false,
                },
            ],
            CubeFace::Down => [
                SideInfo {
                    face: CubeFace::Front,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Right,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Back,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
                SideInfo {
                    face: CubeFace::Left,
                    fixed_idx: N - 1,
                    fixed_idx_is_row: true,
                    flip_varying_idx: false,
                },
            ],
        };
        const NUM_FACES: usize = 4;
        let shift = match turn.dir {
            Direction::Clockwise => 1,
            Direction::Double => 2,
            Direction::Counterclockwise => NUM_FACES - 1,
        };
        for varying_idx in 0..N {
            let mut colors = [Color::White; NUM_FACES];
            for (i, side_info) in side_infos.iter().enumerate() {
                let coord = RefCube::<N>::build_coord(varying_idx, side_info);
                colors[i] = self.0[side_info.face as usize].at(&coord);
            }
            for (i, color) in colors.iter().enumerate() {
                let next_side_info = &side_infos[(i + shift) % NUM_FACES];
                let next_coord =
                    RefCube::<N>::build_coord(varying_idx, next_side_info);
                *self.0[next_side_info.face as usize].at_mut(&next_coord) =
                    *color;
            }
        }
    }

    fn get_new_coord(coord: &Coord, dir: Direction) -> Coord {
        let Coord { row, col } = *coord;
        match dir {
            Direction::Clockwise => Coord {
                row: col,
                col: N - 1 - row,
            },
            Direction::Double => Coord {
                row: N - 1 - row,
                col: N - 1 - col,
            },
            Direction::Counterclockwise => Coord {
                row: N - 1 - col,
                col: row,
            },
        }
    }

    fn rotate_face(&mut self, turn: &Turn) {
        let face = &mut self.0[turn.face as usize];
        let old_face = face.clone();
        for row in 0..N {
            for col in 0..N {
                let coord = Coord { row, col };
                *face.at_mut(&RefCube::<N>::get_new_coord(&coord, turn.dir)) =
                    old_face.at(&coord);
            }
        }
    }

    pub fn is_solved(&self) -> bool {
        for face in &self.0 {
            if !face.is_solved() {
                return false;
            }
        }
        true
    }
}

impl<const N: usize> Cube<N> for RefCube<N> {
    fn from_colors(face_to_color: &[(CubeFace, Color); NUM_FACES]) -> Self {
        let mut cube = RefCube::default();
        for (face, color) in face_to_color {
            cube.0[*face as usize].fill(*color);
        }
        cube
    }

    fn parse_str(mut s: &str) -> Result<Self, ParsingErr> {
        let mut cube = RefCube::new();
        for face in 0..NUM_FACES {
            for row in 0..N {
                for col in 0..N {
                    let (color, rem) = Color::parse_str(s)?;
                    *cube.0[face].at_mut(&Coord { row, col }) = color;
                    s = rem;
                }
            }
        }
        Ok(cube)
    }

    fn apply_turn(&mut self, turn: &Turn) {
        assert!(turn.num_layers == 1);
        self.cycle_sides(turn);
        self.rotate_face(turn);
    }
}

impl<const N: usize> Display for RefCube<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.display_faces(f, &[None, Some(CubeFace::Up)])?;
        self.display_faces(
            f,
            &[
                Some(CubeFace::Left),
                Some(CubeFace::Front),
                Some(CubeFace::Right),
                Some(CubeFace::Back),
            ],
        )?;
        self.display_faces(f, &[None, Some(CubeFace::Down)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t_perm() {
        let turns = Turns::parse_str("RUR'U'R'FR2U'R'U'RUR'F'").unwrap();
        let mut cube = RefCube::<3>::new();
        cube.apply_turns(&turns);
        let cube_ref = RefCube::<3>::parse_str(
            "WWWWWWWWWOROOOOOOOGGRGGGGGGBOGRRRRRRRBBBBBBBBYYYYYYYYY",
        )
        .unwrap();
        assert_eq!(cube, cube_ref);
    }

    #[test]
    fn test_cube_in_a_cube() {
        let turns = Turns::parse_str(
            "FLFU'RUF2L2U'L'BD'B'L2UB2R'DRD'R'DRUR'D'RDR'D'RU'B2",
        )
        .unwrap();
        let mut cube = RefCube::<3>::new();
        cube.apply_turns(&turns);
        let cube_ref = RefCube::<3>::parse_str(
            "GGGGWWGWGYYYOOYYOYRGRRGGRRRWRWRRWWWWOOOOBBOBOBBBYYBBYB",
        )
        .unwrap();
        assert_eq!(cube, cube_ref);
    }
}
