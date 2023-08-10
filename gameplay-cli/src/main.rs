use std::io;
use reqwest::{Error, Response};

use gameplay::games::connect4::{Action, Connect4};
use gameplay::games::{GameState, GameStatus};

mod tui;

enum Agent {
    Human,
    Local,
}

async fn play(blue: Agent, red: Agent) -> io::Result<()> {
    let client = reqwest::Client::new();

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
            Agent::Local => {
                tui::show_connect4(&state, false)?;
                // Query the agent for an action
                let resp = client.post("http://localhost:8000/")
                    .json(&state)
                    .send()
                    .await;
                match resp {
                    Ok(resp) => {
                        let action = resp.json::<Action>().await.unwrap();
                        action
                    }
                    Err(err) => {
                        tui::show_error(err)?;
                        while tui::read_char()? != 'q' {}
                        return Ok(());
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

#[tokio::main]
async fn main() -> io::Result<()> {
    tui::setup()?;
    loop {
        tui::main_menu()?;
        match tui::read_char()? {
            '1' => play(Agent::Human, Agent::Human).await?,
            '2' => play(Agent::Human, Agent::Local).await?,
            'q' => {
                break;
            }
            _ => (),
        }
    }
    tui::cleanup()
}
