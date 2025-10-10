use crate::cube::Cube;
use crate::turn::{Turn, Turns};

use std::collections::{HashMap, VecDeque};
use std::io;
use std::io::Write;
use std::marker::PhantomData;

pub struct Solver<const N: usize, C: Cube<N>> {
    cube_type: PhantomData<C>
}
// TODO: I cannot figure out how to move this type alias inside the impl block
// of solver without the compiler complaining. Something to do with "inherent
// associated types"??? Cannot define a type alias outside, either, because
// this is generic over N. Currently, we just puke this ugly type all over the
// place. Find a way to make it work would be better.
// type SolverStateMap = HashMap<Cube<N>, Option<Turn>>;

impl<const N: usize, C: Cube<N>> Solver<N, C> {
    fn sol_len_upper_bound() -> usize {
        match N {
            2 => 11,
            3 => 20,
            _ => todo!("Fill in the upper bound for the other sized cubes"),
        }
    }

    fn get_all_states_within(
        cube: &C,
        len: usize,
    ) -> HashMap<C, Option<Turn>> {
        let all_turns = Turn::all_turns::<N>();
        // A map from a cube state to a `Turn`, representing the previous turn
        // that brought us to this state. The initial state does not have a
        // previous turn.
        let mut cube_states = HashMap::new();
        // A queue of (cube state, len), where len is the number of turns it
        // takes to get from the initial state to this state
        let mut queue = VecDeque::new();
        cube_states.insert(cube.clone(), None);
        queue.push_back((cube.clone(), 0));
        let mut cube_states_processed = 0;
        while !queue.is_empty() {
            print!("\rCube states processed: {}", cube_states_processed);
            cube_states_processed += 1;
            io::stdout().flush().expect("Flush error");

            let (cube, cur_len) = queue.pop_front().unwrap();
            for turn in all_turns.iter() {
                let mut cube = cube.clone();
                cube.apply_turn(turn);
                if cur_len < len && !cube_states.contains_key(&cube) {
                    queue.push_back((cube.clone(), cur_len + 1));
                    cube_states.insert(cube, Some(turn.clone()));
                }
            }
        }
        println!();
        cube_states
    }

    fn get_turns_from_state(
        cube: &C,
        map: &HashMap<C, Option<Turn>>,
    ) -> Turns {
        let mut turns = Vec::new();
        let mut cube = cube.clone();
        loop {
            match map.get(&cube).unwrap() {
                Some(turn) => {
                    let turn = turn.inverse();
                    cube.apply_turn(&turn);
                    turns.push(turn);
                }
                None => {
                    return Turns(turns);
                }
            }
        }
    }

    fn get_turns_to_state(
        cube: &C,
        map: &HashMap<C, Option<Turn>>,
    ) -> Turns {
        let mut turns = Solver::get_turns_from_state(cube, map).0;
        turns.reverse();
        for turn in &mut turns {
            *turn = turn.inverse();
        }
        Turns(turns)
    }

    pub fn solve(cube: &C) -> Turns {
        let upper_bound = Solver::<N, C>::sol_len_upper_bound();
        let len = if upper_bound % 2 == 0 {
            upper_bound / 2
        } else {
            (upper_bound + 1) / 2
        };
        let solved_cube = cube.from_corner_colors();
        let from_initial = Solver::get_all_states_within(cube, len);
        let from_solved = Solver::get_all_states_within(&solved_cube, len);
        let mut best_sol: Option<Turns> = None;
        for cube in from_initial.keys() {
            if !from_solved.contains_key(cube) {
                continue;
            }
            let mut turns_from_initial =
                Solver::get_turns_to_state(cube, &from_initial).0;
            let turns_to_solved =
                Solver::get_turns_from_state(cube, &from_solved).0;
            let cur_sol_len = turns_from_initial.len() + turns_to_solved.len();
            if let Some(ref cur_best_sol) = best_sol
                && cur_best_sol.0.len() <= cur_sol_len
            {
                continue;
            }
            turns_from_initial.extend(turns_to_solved);
            best_sol = Some(Turns(turns_from_initial));
        }
        best_sol.unwrap()
    }
}
