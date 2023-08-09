use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Match, MatchResult, MatchStatus};

pub const ROWS: usize = 6;
pub const COLS: usize = 7;

fn check4(a: Option<usize>, b: Option<usize>, c: Option<usize>, d: Option<usize>) -> Option<usize> {
    match (a, b, c, d) {
        (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => Some(i),
        _ => None,
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Column must be between 0 and 6. Got `{0}`.")]
    UnknownColumn(usize),
    #[error("Column `{0}` is full.")]
    FullColumn(usize),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    pub column: usize,
}

type State = Vec<Option<usize>>;

#[derive(Debug, Clone)]
pub struct Connect4 {
    board: State,
    next_player: usize,
}

impl Connect4 {
    fn get(&self, col: usize, row: usize) -> Option<usize> {
        self.board[col * ROWS + row]
    }

    fn set(&mut self, col: usize, row: usize, val: Option<usize>) {
        self.board[col * ROWS + row] = val;
    }

    fn _valid_action(&self, action: &Action) -> bool {
        if action.column >= COLS {
            return false;
        }
        self.get(action.column, ROWS - 1).is_none()
    }

    #[allow(clippy::identity_op)]
    fn _status(&self) -> MatchStatus {
        // Check vertical wins
        for col in 0..COLS {
            for row in 0..3 {
                if let Some(player) = check4(
                    self.get(col, row + 0),
                    self.get(col, row + 1),
                    self.get(col, row + 2),
                    self.get(col, row + 3),
                ) {
                    return MatchStatus::Over(MatchResult::Winner {
                        winning_player: player,
                    });
                }
            }
        }

        // Check horizontal wins
        for row in 0..ROWS {
            for col in 0..4 {
                if let Some(player) = check4(
                    self.get(col + 0, row),
                    self.get(col + 1, row),
                    self.get(col + 2, row),
                    self.get(col + 3, row),
                ) {
                    return MatchStatus::Over(MatchResult::Winner {
                        winning_player: player,
                    });
                }
            }
        }

        // Check diagonal up wins
        for col in 0..4 {
            for row in 0..3 {
                if let Some(player) = check4(
                    self.get(col + 0, row + 0),
                    self.get(col + 1, row + 1),
                    self.get(col + 2, row + 2),
                    self.get(col + 3, row + 3),
                ) {
                    return MatchStatus::Over(MatchResult::Winner {
                        winning_player: player,
                    });
                }
            }
        }

        // Check diagonal down wins
        for col in 0..4 {
            for row in 3..6 {
                if let Some(player) = check4(
                    self.get(col + 0, row - 0),
                    self.get(col + 1, row - 1),
                    self.get(col + 2, row - 2),
                    self.get(col + 3, row - 3),
                ) {
                    return MatchStatus::Over(MatchResult::Winner {
                        winning_player: player,
                    });
                }
            }
        }

        // Check for tie
        for col in 0..COLS {
            if self.get(col, ROWS - 1).is_none() {
                return MatchStatus::InProgress {
                    next_player: self.next_player,
                };
            }
        }

        MatchStatus::Over(MatchResult::Tie)
    }

    fn _apply_action(&mut self, action: &Action) -> Result<MatchStatus, Error> {
        if action.column >= COLS {
            return Err(Error::UnknownColumn(action.column));
        }
        for row in 0..ROWS {
            if self.get(action.column, row).is_none() {
                self.set(action.column, row, Some(self.next_player));
                self.next_player = (self.next_player + 1) % 2;
                return Ok(self._status());
            }
        }
        Err(Error::FullColumn(action.column))
    }
}

impl Default for Connect4 {
    fn default() -> Self {
        Self {
            board: vec![None; ROWS * COLS],
            next_player: 0,
        }
    }
}

impl Match for Connect4 {
    type Error = Error;
    type Action = Action;
    type State = State;

    fn valid_action(&self, action: &Self::Action) -> bool {
        self._valid_action(action)
    }
    fn status(&self) -> MatchStatus {
        self._status()
    }
    fn apply_action(&mut self, action: &Self::Action) -> Result<MatchStatus, Self::Error> {
        self._apply_action(action)
    }
    fn state(&self) -> &Self::State {
        &self.board
    }
}
