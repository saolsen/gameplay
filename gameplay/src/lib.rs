mod games;

use std::error::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchResult {
    Winner { winning_player: usize }, // Index of the winning player
    Tie,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchStatus {
    InProgress { next_player: usize }, // Index of the player who's turn it is
    Over(MatchResult),
}

pub trait Match: Default {
    type Error: Error;
    type Action: Serialize + for<'a> Deserialize<'a>;
    type State: Serialize + for<'a> Deserialize<'a>;

    fn valid_action(&self, action: &Self::Action) -> bool;
    fn status(&self) -> MatchStatus;
    fn apply_action(&mut self, action: &Self::Action) -> Result<MatchStatus, Self::Error>;
    fn state(&self) -> &Self::State;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }
}
