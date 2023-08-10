pub mod connect4;

use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Winner { winning_player: usize }, // Index of the winning player
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress { next_player: usize }, // Index of the player who's turn it is
    Over {result: GameResult},
}

pub trait GameState: Default + Clone + Serialize + for<'a> Deserialize<'a> {
    type Error: Error;
    type Action: Serialize + for<'a> Deserialize<'a>;

    fn valid_action(&self, action: &Self::Action) -> bool;
    fn status(&self) -> GameStatus;
    /// Apply an action. Mutates the game state and returns it's status.
    fn apply_action(&mut self, action: &Self::Action) -> Result<GameStatus, Self::Error>;
}
