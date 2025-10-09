use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Yellow,
    Red,
    Orange,
    Blue,
    Green,
}

#[derive(Debug)]
pub enum ParsingErr {
    InvalidColor,
    InvalidLen,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Yellow,
            Color::Yellow => Color::White,
            Color::Red => Color::Orange,
            Color::Orange => Color::Red,
            Color::Blue => Color::Green,
            Color::Green => Color::Blue,
        }
    }

    pub fn parse_str(s: &str) -> Result<(Color, &str), ParsingErr> {
        let mut chars = s.chars();
        let color = match chars.next() {
            None => {
                return Err(ParsingErr::InvalidLen);
            }
            Some('W') => Color::White,
            Some('Y') => Color::Yellow,
            Some('R') => Color::Red,
            Some('O') => Color::Orange,
            Some('B') => Color::Blue,
            Some('G') => Color::Green,
            _ => {
                return Err(ParsingErr::InvalidColor);
            }
        };
        Ok((color, chars.as_str()))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Color::White => "⬜",
                Color::Yellow => "🟨",
                Color::Red => "🟥",
                Color::Orange => "🟧",
                Color::Blue => "🟦",
                Color::Green => "🟩",
            }
        )
    }
}
