#![allow(clippy::identity_op)]

use crate::types;

const ROWS: usize = 6;
const COLS: usize = 7;

pub fn take_turn(
    state: &mut types::Connect4State,
    action: &types::Connect4Action,
    player: usize,
) -> Result<(), String> {
    for row in 0..ROWS {
        let cell = &mut state.board[action.column * ROWS + row];
        if cell.is_none() {
            *cell = Some(player);
            break;
        }
        if row == ROWS - 1 {
            return Err("Column is full".to_owned());
        }
    }
    Ok(())
}

pub enum Connect4Result {
    Winner(usize),
    Tie,
    InProgress,
}

// Check for a winner

pub fn check(state: &types::Connect4State) -> Connect4Result {
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
                    return Connect4Result::Winner(i)
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
                    return Connect4Result::Winner(i)
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
                    return Connect4Result::Winner(i)
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
                    return Connect4Result::Winner(i)
                }
                _ => (),
            }
        }
    }

    // Check for tie
    for col in 0..COLS {
        if state.board[col * ROWS + ROWS - 1].is_none() {
            return Connect4Result::InProgress;
        }
    }

    Connect4Result::Tie
}
