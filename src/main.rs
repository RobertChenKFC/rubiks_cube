mod color;
mod cube;
mod cube2;
mod cube3;
mod cube3_solver;
mod face;
mod row;
mod solver;
mod turn;

use cube::{Cube, DisplayCube, RefCube};
use cube2::Cube2;
use cube3::Cube3;
use cube3_solver::{CompactCube3, Cube3Solver};
use solver::{NaiveSolver, Solver};
use turn::{Turn, Turns};

use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::mem::size_of;
use std::time::Instant;

fn test_cube2<const N: usize, C: Cube<N>>(title: &str, turns: &Turns) {
    let mut cube = C::new();
    cube.apply_turns(turns);
    println!("{}", DisplayCube::new(&cube));

    let now = Instant::now();
    let solver = NaiveSolver::new();
    let turns = solver.solve(&cube);
    let elapsed = now.elapsed().as_micros();
    println!("Solved! Solution: {}", turns);
    println!("{}: Took {} us", title, elapsed);
}

fn test_cube3(solver: &Cube3Solver, title: &str, turns: &Turns) {
    let mut cube = Cube3::new();
    cube.apply_turns(turns);
    println!("{}", DisplayCube::new(&cube));

    let now = Instant::now();
    let turns = solver.solve(&cube);
    let elapsed = now.elapsed().as_micros();
    println!("Solved! Solution: {}", turns);
    println!("{}: Took {} us", title, elapsed);
}

fn main() -> io::Result<()> {
    // Test cases:
    // 1. T-perm: RUR'U'R'FR2U'R'U'RUR'F'
    // 2. Y-perm: FRU'R'U'RUR'F'RUR'U'R'FRF'
    let cube3_solver = Cube3Solver::new("table_7.bin");
    /*
    loop {
        print!("Enter turns here: ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        let turns = Turns::parse_str(line.trim()).unwrap();
        // test_cube2::<2, RefCube<2>>("Ref cube", &turns);
        // test_cube2::<2, Cube2>("Cube2", &turns);
        test_cube3(&cube3_solver, "Cube3", &turns);
    }
    */
    Ok(())
}
