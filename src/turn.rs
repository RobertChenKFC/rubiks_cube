use crate::cube::NUM_FACES;

use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CubeFace {
    Up = 0,
    Left = 1,
    Front = 2,
    Right = 3,
    Back = 4,
    Down = 5,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Direction {
    Clockwise,
    Double,
    Counterclockwise,
}

pub const NUM_DIRS: usize = 3;
pub const ALL_DIRS: [Direction; NUM_DIRS] = [
    Direction::Clockwise,
    Direction::Double,
    Direction::Counterclockwise,
];

#[derive(Clone, PartialEq, Debug)]
pub struct Turn {
    pub face: CubeFace,
    pub dir: Direction,
    pub num_layers: usize,
}

#[derive(Debug)]
pub enum ParsingErr {
    EmptyString,
    InvalidFace,
}

pub const ALL_FACES: [CubeFace; NUM_FACES] = [
    CubeFace::Up,
    CubeFace::Left,
    CubeFace::Front,
    CubeFace::Right,
    CubeFace::Back,
    CubeFace::Down,
];

const RUF_FACES: [CubeFace; 3] =
    [CubeFace::Right, CubeFace::Up, CubeFace::Front];

impl Turn {
    pub fn parse_str(s: &str) -> Result<(Turn, &str), ParsingErr> {
        let mut chars = s.chars();
        let face = match chars.next() {
            Some('U') => CubeFace::Up,
            Some('L') => CubeFace::Left,
            Some('F') => CubeFace::Front,
            Some('R') => CubeFace::Right,
            Some('B') => CubeFace::Back,
            Some('D') => CubeFace::Down,
            Some(_) => return Err(ParsingErr::EmptyString),
            _ => return Err(ParsingErr::InvalidFace),
        };
        let mut turn = Turn {
            face,
            dir: Direction::Clockwise,
            num_layers: 1,
        };
        if let Some(c) = chars.next() {
            match c {
                '\'' => turn.dir = Direction::Counterclockwise,
                '2' => turn.dir = Direction::Double,
                _ => (),
            }
        }
        let mut chars = s.chars();
        chars.next();
        if turn.dir != Direction::Clockwise {
            chars.next();
        }
        return Ok((turn, chars.as_str()));
    }

    fn all_turns_impl<const N: usize>(all_faces: &[CubeFace]) -> Turns {
        let mut turns = Vec::new();
        for face in all_faces {
            for dir in [
                Direction::Clockwise,
                Direction::Double,
                Direction::Counterclockwise,
            ] {
                for num_layers in 1..=N / 2 {
                    turns.push(Turn {
                        face: *face,
                        dir,
                        num_layers,
                    });
                }
            }
        }
        Turns(turns)
    }

    pub fn all_turns<const N: usize>() -> Turns {
        Turn::all_turns_impl::<N>(&ALL_FACES)
    }

    pub fn all_required_turns<const N: usize>() -> Turns {
        let all_faces: &[CubeFace] = if N == 2 {
            // 2x2 cubes only need the R, F, U faces to reach any cube state
            &RUF_FACES
        } else {
            &ALL_FACES
        };
        Turn::all_turns_impl::<N>(all_faces)
    }

    pub fn inverse(&self) -> Turn {
        Turn {
            face: self.face,
            dir: match self.dir {
                Direction::Clockwise => Direction::Counterclockwise,
                Direction::Double => Direction::Double,
                Direction::Counterclockwise => Direction::Clockwise,
            },
            num_layers: self.num_layers,
        }
    }
}

impl Display for Turn {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.face {
                CubeFace::Up => "U",
                CubeFace::Left => "L",
                CubeFace::Front => "F",
                CubeFace::Right => "R",
                CubeFace::Back => "B",
                CubeFace::Down => "D",
            }
        )?;
        write!(
            f,
            "{}",
            match self.dir {
                Direction::Clockwise => "",
                Direction::Double => "2",
                Direction::Counterclockwise => "'",
            }
        )
    }
}

pub struct Turns(pub Vec<Turn>);

impl Turns {
    pub fn parse_str(mut s: &str) -> Result<Turns, ParsingErr> {
        let mut turns = Vec::new();
        while !s.is_empty() {
            let (turn, rem) = Turn::parse_str(s)?;
            turns.push(turn);
            s = rem;
        }
        Ok(Turns(turns))
    }

    pub fn iter(&self) -> slice::Iter<'_, Turn> {
        self.0.iter()
    }
}

impl Display for Turns {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for turn in self.iter() {
            write!(f, "{}", turn)?;
        }
        Ok(())
    }
}
