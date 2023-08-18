use gameplay::games::connect4::{Action, Connect4, COLS};
use gameplay::games::{GameResult, GameState, GameStatus};
use rand::Rng;

pub fn agent(state: &Connect4) -> Action {
    let player = match state.status() {
        GameStatus::InProgress { next_player } => next_player,
        _ => panic!("Game is over."),
    };

    // For each action we could take, simulate multiple random games from the resulting state.
    // Keep track of the number of wins for each action.
    // Pick the action with the highest win rate.
    let mut max_score = f64::MIN;
    let mut best_action = Action { column: 0 };

    for col in 0..COLS {
        let action = Action { column: col };
        if state.valid_action(&action) {
            let score = score_action(state, player, &action);
            if score > max_score {
                max_score = score;
                best_action = action;
            }
        }
    }

    best_action
}

#[cfg(debug_assertions)]
const SIMULATIONS: i32 = 1_000;

#[cfg(not(debug_assertions))]
const SIMULATIONS: i32 = 10_000;

fn score_action(current_state: &Connect4, player: usize, action: &Action) -> f64 {
    let mut rng = rand::thread_rng();

    // Create a new match with the action applied.
    let mut next_state = current_state.clone();
    next_state.apply_action(action).unwrap();

    // Simulate random games from this state.
    let mut score = 0;
    for _ in 0..SIMULATIONS {
        let mut sim = next_state.clone();
        let mut status = sim.status();
        loop {
            match status {
                GameStatus::Over { result } => {
                    match result {
                        GameResult::Winner { winning_player } => {
                            if winning_player == player {
                                score += 1;
                            } else {
                                score -= 1;
                            }
                        }
                        GameResult::Tie => {}
                        GameResult::Error => {}
                    }
                    break;
                }
                GameStatus::InProgress { .. } => {
                    let action = loop {
                        let column = rng.gen_range(0..COLS);
                        let action = Action { column };
                        if sim.valid_action(&action) {
                            break action;
                        }
                    };
                    status = sim.apply_action(&action).unwrap();
                }
            }
        }
    }
    score as f64 / SIMULATIONS as f64
}
