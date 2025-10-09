use crate::color::Color;
use crate::row::Row;

use std::fmt::{Formatter, Result};

pub struct Coord {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Face<const N: usize>([Row<N>; N]);

impl<const N: usize> Face<N> {
    pub fn new(c: Color) -> Face<N> {
        Face([Row::new(c); N])
    }

    pub fn at_mut<'a>(&'a mut self, coord: &Coord) -> &'a mut Color {
        self.0[coord.row].at_mut(coord.col)
    }

    pub fn at(&self, coord: &Coord) -> Color {
        self.0[coord.row].at(coord.col)
    }

    pub fn fill(&mut self, color: Color) {
        for row in &mut self.0 {
            row.fill(color);
        }
    }

    pub fn display_row(&self, f: &mut Formatter, row_idx: usize) -> Result {
        write!(f, "{}", self.0[row_idx])
    }

    pub fn is_solved(&self) -> bool {
        let color = self.at(&Coord { row: 0, col: 0 });
        for row in 0..N {
            for col in 0..N {
                if self.at(&Coord { row, col }) != color {
                    return false;
                }
            }
        }
        return true;
    }
}
