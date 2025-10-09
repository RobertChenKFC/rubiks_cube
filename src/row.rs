use crate::color::Color;

use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct Row<const N: usize>([Color; N]);

impl<const N: usize> Row<N> {
    pub fn new(c: Color) -> Row<N> {
        Row([c; N])
    }

    pub fn at_mut<'a>(&'a mut self, col: usize) -> &'a mut Color {
        &mut self.0[col]
    }

    pub fn at(&self, col: usize) -> Color {
        self.0[col]
    }

    pub fn fill(&mut self, color: Color) {
        for cell in &mut self.0 {
            *cell = color;
        }
    }
}

impl<const N: usize> Display for Row<N> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        for c in &self.0 {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}
