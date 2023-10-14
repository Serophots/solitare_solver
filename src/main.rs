mod solver;
mod solitare;

use crate::solitare::solitare::{Game};
use crate::solver::solve;

fn main() {
    let game = Box::new(Game::new());

    solve(game);
}