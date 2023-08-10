use std::io;

use gameplay::games::connect4::{Action, Connect4};
use gameplay::games::{GameState, GameStatus};

mod tui;

enum Agent {
    Human,
}

fn play(blue: Agent, red: Agent) -> io::Result<()> {
    let mut state = Connect4::default();
    let mut status = state.status();
    while let GameStatus::InProgress { next_player } = status {
        let agent = match next_player {
            0 => &blue,
            1 => &red,
            _ => unreachable!(),
        };

        let action = match agent {
            Agent::Human => {
                tui::show_connect4(&state, true)?;
                'turn: loop {
                    let c = tui::read_char()?;
                    if c == 'q' {
                        return Ok(());
                    }
                    if let Some(c) = c.to_digit(10) {
                        if c == 0 || c > 7 {
                            continue 'turn;
                        }
                        let action = Action {
                            column: (c - 1) as usize,
                        };
                        if state.valid_action(&action) {
                            break 'turn action;
                        }
                    }
                }
            }
        };
        status = state.apply_action(&action).unwrap();
    }
    tui::show_connect4(&state, false)?;
    while tui::read_char()? != 'q' {}
    Ok(())
}

fn main() -> io::Result<()> {
    tui::setup()?;
    loop {
        tui::main_menu()?;
        match tui::read_char()? {
            '1' => play(Agent::Human, Agent::Human)?,
            'q' => {
                break;
            }
            _ => (),
        }
    }
    tui::cleanup()
}
