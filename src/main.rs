mod color;
mod cube;
mod face;
mod row;
mod solver;
mod turn;

use cube::{Cube, RefCube};
use solver::Solver;
use turn::Turns;

use std::io;
use std::io::Write;

fn main() -> io::Result<()> {
    // Test cases:
    // 1. T-perm: RUR'U'R'FR2U'R'U'RUR'F'
    // 2. Y-perm: FRU'R'U'RUR'F'RUR'U'R'FRF'
    loop {
        print!("Enter turns here: ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let turns = Turns::parse_str(line.trim()).unwrap();

        let mut cube: RefCube<2> = RefCube::new();
        cube.apply_turns(&turns);
        println!("{}", cube);

        let turns = Solver::solve(&cube);
        print!("Solved! Solution: {}", turns);
        println!();
    }
}
