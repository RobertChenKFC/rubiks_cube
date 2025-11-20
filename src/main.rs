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
use cube3_solver::{
    CompactCube3, Cube3Solver, CubeHasher, RufCube3, SerializableCube3,
};
use solver::{NaiveSolver, Solver};
use turn::{Turn, Turns};

use std::cmp;
use std::collections::HashMap;
use std::hash::{BuildHasher, RandomState};
use std::io;
use std::io::Write;
use std::mem::size_of;
use std::time::Instant;

fn test_cube2<const N: usize, C: Cube<N>>(title: &str, turns: &Turns) {
    let mut cube = C::new();
    cube.apply_turns(turns);
    println!("{}", DisplayCube::new(&cube));

    let now = Instant::now();
    let solver: NaiveSolver<_, _> = NaiveSolver::new();
    let turns = solver.solve(&cube);
    let elapsed = now.elapsed().as_micros();
    println!("Solved! Solution: {}", turns);
    println!("{}: Took {} us", title, elapsed);
}

fn test_cube3<
    const N: usize,
    C: SerializableCube3<N>,
    S: BuildHasher + Clone + Default,
>(
    solver: &Cube3Solver<N, C, S>,
    title: &str,
    turns: &Turns,
) {
    let mut cube = Cube3::new();
    cube.apply_turns(turns);
    println!("{}", DisplayCube::new(&cube));

    let now = Instant::now();
    let turns = solver.solve(&cube);
    let elapsed = now.elapsed().as_micros();
    println!("Solved! Solution: {}", turns);
    println!("{}: Took {} us", title, elapsed);
}

fn compute_collisions<S: BuildHasher + Default + Clone>() -> usize {
    let hasher = S::default();
    let mut table = HashMap::new();
    let mut max_cnt = 0;
    for (cube3, _) in
        Cube3Solver::<_, RufCube3>::load_table_elems("table_10.bin")
    {
        let hash = hasher.hash_one(cube3);
        max_cnt = cmp::max(
            max_cnt,
            match table.get_mut(&hash) {
                Some(cnt) => {
                    *cnt += 1;
                    *cnt
                }
                None => {
                    table.insert(hash, 1);
                    1
                }
            },
        );
    }
    let mut histogram = HashMap::new();
    for (_, cnt) in table {
        match histogram.get_mut(&cnt) {
            Some(val) => *val += 1,
            None => {
                histogram.insert(cnt, 1);
            }
        }
    }
    for (cnt, val) in histogram {
        println!("{} hashes with {} collisions", val, cnt);
    }
    max_cnt
}

fn load_into_vec() {
    let now = Instant::now();
    let mut cubes = Vec::new();
    for (cube3, _) in
        Cube3Solver::<_, RufCube3>::load_table_elems("table_10.bin")
    {
        cubes.push(cube3);
    }
    let elapsed = now.elapsed().as_micros();
    println!("Load into vec took {} us", elapsed);

    let now = Instant::now();
    cubes.sort_unstable();
    let elapsed = now.elapsed().as_micros();
    println!("Sorted in {} us", elapsed);

    let now = Instant::now();
    let cube3 = cubes[cubes.len() - 1].clone();
    let mut idx = 0;
    for (i, cube) in cubes.iter().enumerate() {
        if *cube == cube3 {
            idx = i;
            break;
        }
    }
    let elapsed = now.elapsed().as_micros();
    println!("Found cube at {} in {} us", idx, elapsed);
}

fn main() -> io::Result<()> {
    // println!("Random state: {} collisions", compute_collisions::<RandomState>());
    // println!("Cube hasher: {} collisions", compute_collisions::<CubeHasher>());
    // load_into_vec();
    // Cube3Solver::<_, RufCube3>::gen_table("table_10.bin", /*len=*/10);

    // DEBUG
    // Test cases:
    // 1. T-perm: RUR'U'R'FR2U'R'U'RUR'F'
    // 2. Y-perm: FRU'R'U'RUR'F'RUR'U'R'FRF'
    // let cube3_solver = Cube3Solver::<_, CompactCube3>::new("old_tables/table_10.bin");
    // let cube3_solver = Cube3Solver::<_, RufCube3, CubeHasher>::new("table_10.bin");
    let cube3_solver = Cube3Solver::<_, RufCube3>::new("table_10.bin");
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
    Ok(())
}
