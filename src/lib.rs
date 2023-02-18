pub use othello_core_lib::*;
use std::{collections::HashSet, hash::Hash};

pub use ai::*;
pub use arena::*;
pub use game::*;
pub use mixed_player::*;
pub use visual::*;
// use run::*;

pub mod ai;
pub mod arena;
pub mod console;
pub mod elo;
pub mod game;
pub mod mixed_player;
pub mod visual;

// https://stackoverflow.com/questions/46766560/how-to-check-if-there-are-duplicates-in-a-slice
pub fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
