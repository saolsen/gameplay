use std::io;

use clap::{Parser, Subcommand};
use reqwest::Url;

use gameplay::games::connect4::{Action, Connect4};
use gameplay::games::{Game, GameState, GameStatus};

mod tui;

enum Player {
    Human,
    Agent(Url),
}

async fn cli_connect4_match(player1: Player, player2: Player) -> io::Result<()> {
    // Wrap the match in setup/cleanup so we make sure to cleanup on any error.
    tui::setup()?;
    let result = _cli_connect4_match(player1, player2).await;
    tui::cleanup()?;
    result
}

async fn _cli_connect4_match(player1: Player, player2: Player) -> io::Result<()> {
    let client = reqwest::Client::new();

    let mut state = Connect4::default();
    let mut status = state.status();
    while let GameStatus::InProgress { next_player } = status {
        let player = match next_player {
            0 => &player1,
            1 => &player2,
            _ => unreachable!(),
        };

        let action = match player {
            Player::Human => {
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
            Player::Agent(url) => {
                tui::show_connect4(&state, false)?;
                // Query the agent for an action
                let resp = client.post(url.clone()).json(&state).send().await;
                match resp {
                    Ok(resp) => {
                        let action = resp.json::<Action>().await;
                        match action {
                            Ok(action) => {
                                // See if the action is valid.
                                if state.valid_action(&action) {
                                    action
                                } else {
                                    tui::show_error(&format!("Action is invalid: {:?}", action))?;
                                    while tui::read_char()? != 'q' {}
                                    return Ok(());
                                }
                            }
                            Err(err) => {
                                tui::show_error(&err.to_string())?;
                                while tui::read_char()? != 'q' {}
                                return Ok(());
                            }
                        }
                    }
                    Err(err) => {
                        tui::show_error(&err.to_string())?;
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    game: Game,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a match
    Play {
        #[arg(long)]
        player1_url: Option<Url>,
        #[arg(long)]
        player2_url: Option<Url>,
    },
    // Test an agent
    // Test { url: Url },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Cli::parse();

    // Only one game so far, everything assumes connect4.
    assert_eq!(args.game, Game::Connect4);

    match args.command {
        Commands::Play {
            player1_url,
            player2_url,
        } => {
            let player1 = match player1_url {
                Some(url) => Player::Agent(url),
                None => Player::Human,
            };
            let player2 = match player2_url {
                Some(url) => Player::Agent(url),
                None => Player::Human,
            };
            cli_connect4_match(player1, player2).await?;
        } // Commands::Test { url } => {}
    }

    Ok(())
}
