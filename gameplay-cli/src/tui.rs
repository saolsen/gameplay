use std::io;
use std::io::Write;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use crossterm::{
    cursor,
    event::{self, Event},
    execute, queue, style,
    terminal::{self, ClearType},
};

use gameplay::games::connect4::{Action, Connect4, COLS, ROWS};
use gameplay::games::{GameResult, GameState, GameStatus};

pub fn read_char() -> io::Result<char> {
    loop {
        if let Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            kind: KeyEventKind::Press,
            modifiers: _,
            state: _,
        }) = event::read()?
        {
            return Ok(c);
        }
    }
}

pub fn show_error(err: reqwest::Error) -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        style::Print("Connect 4")
    )?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    queue!(
        stdout,
        style::SetForegroundColor(style::Color::Red),
        style::Print("Error: "),
        style::Print(err.to_string()),
        style::ResetColor,
        cursor::MoveToNextLine(1),
        style::Print("hit 'q' to quit"),
        cursor::MoveToNextLine(1),
    )?;

    stdout.flush()
}

const BORDER: &str = "+---+---+---+---+---+---+---+";

pub fn show_connect4(connect4_state: &Connect4, your_turn: bool) -> io::Result<()> {
    let mut stdout = io::stdout();

    // Header
    queue!(
        stdout,
        terminal::Clear(ClearType::All),
        style::SetForegroundColor(style::Color::Black),
        style::SetBackgroundColor(style::Color::White),
        cursor::MoveTo(0, 0)
    )?;

    let status = connect4_state.status();
    match status {
        GameStatus::InProgress { next_player } => {
            match next_player {
                0 => {
                    queue!(
                        stdout,
                        style::SetForegroundColor(style::Color::Blue),
                        style::Print("Blue"),
                    )?;
                }
                1 => {
                    queue!(
                        stdout,
                        style::SetForegroundColor(style::Color::Red),
                        style::Print("Red"),
                    )?;
                }
                _ => unreachable!("Invalid player"),
            }
            queue!(stdout, style::ResetColor, style::Print("'s turn"),)?;
            if your_turn {
                queue!(stdout, style::Print(" (that's you)"))?;
            }
            queue!(stdout, cursor::MoveToNextLine(1))?;
        }
        GameStatus::Over { result } => match result {
            GameResult::Winner { winning_player } => {
                match winning_player {
                    0 => {
                        queue!(
                            stdout,
                            style::SetForegroundColor(style::Color::Blue),
                            style::Print("Blue"),
                        )?;
                    }
                    1 => {
                        queue!(
                            stdout,
                            style::SetForegroundColor(style::Color::Red),
                            style::Print("Red"),
                        )?;
                    }
                    _ => unreachable!("Invalid player"),
                }
                queue!(
                    stdout,
                    style::ResetColor,
                    style::Print(" wins!"),
                    cursor::MoveToNextLine(1)
                )?;
            }
            GameResult::Tie => {
                queue!(
                    stdout,
                    style::Print("It's a Tie"),
                    cursor::MoveToNextLine(1)
                )?;
            }
        },
    }

    if your_turn {
        for column in 0..COLS {
            if connect4_state.valid_action(&Action { column }) {
                queue!(stdout, style::Print(format!("  {} ", column + 1)))?;
            } else {
                queue!(stdout, style::Print("    "))?;
            }
        }
    }
    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        style::Print(BORDER),
        cursor::MoveToNextLine(1)
    )?;
    for row in (0..ROWS).rev() {
        for col in 0..COLS {
            queue!(stdout, style::Print("| "))?;
            match connect4_state.get(col, row) {
                Some(0) => {
                    queue!(
                        stdout,
                        style::SetForegroundColor(style::Color::Blue),
                        style::Print("●"),
                        style::ResetColor
                    )?;
                }
                Some(1) => {
                    queue!(
                        stdout,
                        style::SetForegroundColor(style::Color::Red),
                        style::Print("●"),
                        style::ResetColor
                    )?;
                }
                None => {
                    queue!(stdout, style::Print(" "))?;
                }
                _ => unreachable!("Invalid player"),
            };
            queue!(stdout, style::Print(" "))?;
        }
        queue!(
            stdout,
            style::Print("|"),
            cursor::MoveToNextLine(1),
            style::Print(BORDER),
            cursor::MoveToNextLine(1)
        )?;
    }

    if your_turn {
        queue!(stdout, style::Print("choose a column (1-7) or "),)?;
    }

    queue!(
        stdout,
        style::Print("hit 'q' to quit"),
        cursor::MoveToNextLine(1),
        style::ResetColor
    )?;

    stdout.flush()
}

const MENU: &str = r#"Choose an opponent.

1. Human. (Yourself or the person next to you).
2. Local Agent. (Agent running on port 8000).

Select opponent ('1', '2') or hit 'q' to quit.
"#;

pub fn main_menu() -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(
        stdout,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0),
        style::Print("Connect 4")
    )?;

    queue!(stdout, cursor::MoveToNextLine(1))?;

    for line in MENU.split('\n') {
        queue!(stdout, style::Print(line), cursor::MoveToNextLine(1))?;
    }

    stdout.flush()
}

pub fn setup() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()
}

pub fn cleanup() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()
}
