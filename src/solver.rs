use crate::cube::Cube;
use crate::turn::{Turn, Turns};

use std::collections::{HashMap, VecDeque};
use std::hash::{BuildHasher, RandomState};
use std::io;
use std::io::Write;
use std::marker::PhantomData;

pub trait Solver<const N: usize, C: Cube<N>> {
    fn solve(&self, cube: &C) -> Turns;
}
// TODO: I cannot figure out how to move this type alias inside the impl block
// of solver without the compiler complaining. Something to do with "inherent
// associated types"??? Cannot define a type alias outside, either, because
// this is generic over N. Currently, we just puke this ugly type all over the
// place. Find a way to make it work would be better.
// type SolverStateMap = HashMap<Cube<N>, Option<Turn>>;

pub struct NaiveSolver<
    const N: usize,
    C: Cube<N>,
    S: BuildHasher + Clone + Default = RandomState,
> {
    cube_type: PhantomData<C>,
    hash_builder: S,
}

pub trait CubeTable<const N: usize, C: Cube<N>> {
    fn find(&self, cube: &C) -> Option<Turn>;
}

struct CubeHashTable<
    'a,
    const N: usize,
    C: Cube<N>,
    S: BuildHasher + Clone + Default,
> {
    hashmap: &'a HashMap<C, Option<Turn>, S>,
}

impl<'a, const N: usize, C: Cube<N>, S: BuildHasher + Clone + Default>
    CubeHashTable<'a, N, C, S>
{
    fn new(hashmap: &'a HashMap<C, Option<Turn>, S>) -> Self {
        Self { hashmap }
    }
}

impl<'a, const N: usize, C: Cube<N>, S: BuildHasher + Clone + Default>
    CubeTable<N, C> for CubeHashTable<'a, N, C, S>
{
    fn find(&self, cube: &C) -> Option<Turn> {
        self.hashmap[cube].clone()
    }
}

impl<const N: usize, C: Cube<N>, S: BuildHasher + Clone + Default>
    NaiveSolver<N, C, S>
{
    pub fn new() -> Self {
        Self {
            cube_type: PhantomData,
            hash_builder: S::default(),
        }
    }

    fn sol_len_upper_bound() -> usize {
        match N {
            2 => 11,
            3 => 20,
            _ => todo!("Fill in the upper bound for the other sized cubes"),
        }
    }

    pub fn get_all_states_within(
        &self,
        cube: &C,
        len: usize,
        all_turns: &Turns,
    ) -> HashMap<C, Option<Turn>, S> {
        // A map from a cube state to a `Turn`, representing the previous turn
        // that brought us to this state. The initial state does not have a
        // previous turn.
        let mut cube_states = HashMap::with_hasher(self.hash_builder.clone());
        // A queue of (cube state, len), where len is the number of turns it
        // takes to get from the initial state to this state
        let mut queue = VecDeque::new();
        cube_states.insert(cube.clone(), None);
        queue.push_back((cube.clone(), 0));
        let mut cube_states_processed = 0;
        while !queue.is_empty() {
            cube_states_processed += 1;
            if cube_states_processed % 1000000 == 0 {
                println!(
                    "Processed {} states, {} states stored, {} states in queue",
                    cube_states_processed,
                    cube_states.len(),
                    queue.len()
                );
                io::stdout().flush().expect("Flush error");
            }

            // TODO: DFS is faster and uses less memory. However, it doesn't
            // necessarily find the shortest path, which means that it may miss
            // some cube states that are reachable in `len` turns. Investigate
            // whether it's possible to use DFS.
            // DEBUG
            // let (cube, cur_len) = queue.pop_front().unwrap();
            let (cube, cur_len) = queue.pop_back().unwrap();
            for turn in all_turns.iter() {
                let mut cube = cube.clone();
                cube.apply_turn(turn);
                if cur_len < len && !cube_states.contains_key(&cube) {
                    queue.push_back((cube.clone(), cur_len + 1));
                    cube_states.insert(cube, Some(turn.clone()));
                }
            }
        }
        cube_states
    }

    pub fn get_turns_from_state<T: CubeTable<N, C>>(
        cube: &C,
        map: &T,
    ) -> Turns {
        let mut turns = Vec::new();
        let mut cube = cube.clone();
        loop {
            match map.find(&cube) {
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

    pub fn get_turns_to_state<T: CubeTable<N, C>>(cube: &C, map: &T) -> Turns {
        let mut turns =
            NaiveSolver::<N, C, S>::get_turns_from_state(cube, map).0;
        turns.reverse();
        for turn in &mut turns {
            *turn = turn.inverse();
        }
        Turns(turns)
    }

    pub fn find_turns_to_state(
        &self,
        from_state: &C,
        from_len: usize,
        all_states_to: &HashMap<C, Option<Turn>, S>,
        all_turns: &Turns,
    ) -> Turns {
        let all_states_from =
            self.get_all_states_within(from_state, from_len, all_turns);
        let mut best_sol: Option<Turns> = None;
        for cube in all_states_from.keys() {
            if !all_states_to.contains_key(cube) {
                continue;
            }
            let mut turns_from = NaiveSolver::<N, C, S>::get_turns_to_state(
                cube,
                &CubeHashTable::new(&all_states_from),
            )
            .0;
            let turns_to = NaiveSolver::<N, C, S>::get_turns_from_state(
                cube,
                &CubeHashTable::new(&all_states_to),
            )
            .0;
            let cur_sol_len = turns_from.len() + turns_to.len();
            if let Some(ref cur_best_sol) = best_sol
                && cur_best_sol.0.len() <= cur_sol_len
            {
                continue;
            }
            turns_from.extend(turns_to);
            best_sol = Some(Turns(turns_from));
        }
        best_sol.unwrap()
    }

    pub fn meet_in_the_middle_with_turns(
        &self,
        from_state: &C,
        to_state: &C,
        len: usize,
        all_turns: &Turns,
    ) -> Turns {
        let from_len = len / 2 + len % 2;
        let to_len = len - from_len;
        let all_states_to =
            self.get_all_states_within(to_state, to_len, all_turns);
        self.find_turns_to_state(
            from_state,
            from_len,
            &all_states_to,
            all_turns,
        )
    }

    pub fn meet_in_the_middle(
        &self,
        from_state: &C,
        to_state: &C,
        len: usize,
    ) -> Turns {
        let all_turns = Turn::all_required_turns::<N>();
        self.meet_in_the_middle_with_turns(
            from_state, to_state, len, &all_turns,
        )
    }
}

impl<const N: usize, C: Cube<N>, S: BuildHasher + Clone + Default> Solver<N, C>
    for NaiveSolver<N, C, S>
{
    fn solve(&self, cube: &C) -> Turns {
        let len = NaiveSolver::<N, C, S>::sol_len_upper_bound();
        let solved_cube = cube.from_corner_colors();
        self.meet_in_the_middle(cube, &solved_cube, len)
    }
}
