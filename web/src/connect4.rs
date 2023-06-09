#![allow(clippy::identity_op)]
use thiserror::Error;

use crate::types;

const ROWS: usize = 6;
const COLS: usize = 7;

#[derive(Error, Debug)]
pub enum ActionError {
    #[error("Column must be between 0 and 6. Got `{0}`.")]
    UnknownColumn(usize),
    #[error("Column `{0}` is full.")]
    FullColumn(usize),
}

pub fn apply_action(
    state: &mut types::Connect4State,
    action: &types::Connect4Action,
    player: usize,
) -> Result<(), ActionError> {
    use ActionError::*;
    if action.column >= COLS {
        return Err(UnknownColumn(action.column));
    }
    for row in 0..ROWS {
        let cell = &mut state.board[action.column * ROWS + row];
        if cell.is_none() {
            *cell = Some(player);
            break;
        }
        if row == ROWS - 1 {
            return Err(FullColumn(action.column));
        }
    }
    Ok(())
}

pub enum Connect4Check {
    Winner(usize),
    Tie,
    InProgress,
}

pub fn check(state: &types::Connect4State) -> Connect4Check {
    use Connect4Check::*;
    // Check vertical wins
    for col in 0..COLS {
        for row in 0..3 {
            match (
                state.board[col * ROWS + row + 0],
                state.board[col * ROWS + row + 1],
                state.board[col * ROWS + row + 2],
                state.board[col * ROWS + row + 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Winner(i)
                }
                _ => (),
            }
        }
    }

    // Check horizontal wins
    for row in 0..ROWS {
        for col in 0..4 {
            match (
                state.board[(col + 0) * ROWS + row],
                state.board[(col + 1) * ROWS + row],
                state.board[(col + 2) * ROWS + row],
                state.board[(col + 3) * ROWS + row],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Winner(i)
                }
                _ => (),
            }
        }
    }

    // Check diagonal up wins
    for col in 0..4 {
        for row in 0..3 {
            match (
                state.board[(col + 0) * ROWS + row + 0],
                state.board[(col + 1) * ROWS + row + 1],
                state.board[(col + 2) * ROWS + row + 2],
                state.board[(col + 3) * ROWS + row + 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Winner(i)
                }
                _ => (),
            }
        }
    }

    // Check diagonal down wins
    for col in 0..4 {
        for row in 3..6 {
            match (
                state.board[(col + 0) * ROWS + row - 0],
                state.board[(col + 1) * ROWS + row - 1],
                state.board[(col + 2) * ROWS + row - 2],
                state.board[(col + 3) * ROWS + row - 3],
            ) {
                (Some(i), Some(j), Some(k), Some(l)) if i == j && j == k && k == l => {
                    return Winner(i)
                }
                _ => (),
            }
        }
    }

    // Check for tie
    for col in 0..COLS {
        if state.board[col * ROWS + ROWS - 1].is_none() {
            return InProgress;
        }
    }

    Tie
}
