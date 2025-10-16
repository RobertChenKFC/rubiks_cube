mod color;
mod cube;
mod cube2;
mod face;
mod row;
mod solver;
mod turn;

use cube::{Cube, DisplayCube, RefCube};
use cube2::Cube2;
use solver::Solver;
use turn::Turns;

use std::io;
use std::io::Write;
use std::time::Instant;

fn test_cube<const N: usize, C: Cube<N>>(title: &str, turns: &Turns) {
    let mut cube = C::new();
    cube.apply_turns(turns);
    println!("{}", DisplayCube::new(&cube));

    let now = Instant::now();
    let turns = Solver::solve(&cube);
    let elapsed = now.elapsed().as_micros();
    println!("Solved! Solution: {}", turns);
    println!("{}: Took {} us", title, elapsed);
}

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
        test_cube::<2, RefCube<2>>("Ref cube", &turns);
        test_cube::<2, Cube2>("Cube2", &turns);
    }
}
