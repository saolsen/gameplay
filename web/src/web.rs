use crate::forms::create_match;
use crate::types::Connect4Action;
use crate::{config, migrations, templates, types};
use askama_axum::IntoResponse as _;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Form;
use jwt_simple::algorithms::RS256PublicKey;
use rusqlite::{params, OptionalExtension};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{info_span, Instrument};

#[derive(Debug)]
pub struct AppState {
    pub key: RS256PublicKey,
    pub pool: types::Pool,
}

impl AppState {
    pub fn new() -> Self {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(&*config::DB).with_init(|c| {
            c.execute_batch(
                r#"
                    PRAGMA journal_mode = wal;
                    PRAGMA synchronous = normal;
                    PRAGMA foreign_keys = on;
                "#,
            )
        });
        let pool = types::Pool::new(manager).unwrap();
        {
            let mut conn = pool.get().unwrap();
            migrations::migrate(&mut conn).unwrap();
        }
        let key = RS256PublicKey::from_pem(&config::CLERK_PUB_ENCRYPTION_KEY).unwrap();
        Self { pool, key }
    }
}

pub const CSS: &str = include_str!("../output.css");

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tracing::instrument]
pub async fn refresh() -> impl IntoResponse {
    let refresh = templates::Refresh {
        clerk_pub_api_key: &config::CLERK_PUB_API_KEY,
    };
    (StatusCode::OK, refresh.into_response())
}

#[tracing::instrument(skip(web_layout, state))]
pub async fn root<'a>(
    web_layout: templates::WebLayout<'a>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pool = state.pool.clone();
    let _result = tokio::task::spawn_blocking(move || {
        let conn = pool.get().unwrap();
        let res: String = conn
            .query_row(r#"select sqlite_version() as version"#, [], |row| {
                row.get(0)
            })
            .unwrap();
        res
    })
    .await
    .unwrap();

    web_layout.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppIndex {
        _layout: app_layout,
        create_match: create_match::CreateMatchForm::default(&auth_user),
    };
    index.into_response()
}

// note: I'm doing everything in-line to start. It's very tbd what things go in which modules
// until I see it all laid out. Luckily refactoring is so easy is rust.
#[tracing::instrument(skip(state))]
pub async fn connect4_create_match<'a>(
    auth_user: types::UserRecord,
    State(state): State<Arc<AppState>>,
    Form(form): Form<create_match::CreateMatchFormData>,
) -> impl IntoResponse {
    tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get().unwrap();

        let mut blue_error = None;
        let mut red_error = None;

        // Validate the selections
        let blue_select = match form.player_type_1.as_str() {
            "me" => {
                if form.player_name_1 != auth_user.username {
                    blue_error = Some("Me must be you.".to_string());
                }
                create_match::CreateMatchFormSelects {
                    i: 1,
                    options: create_match::CreateMatchOptions::me(&auth_user),
                    selected: Some(auth_user.username.clone()),
                }
            }
            "user" => {
                if form.player_type_1 == auth_user.username {
                    blue_error = Some("Select 'me' for yourself.".to_string());
                }
                create_match::CreateMatchFormSelects {
                    i: 1,
                    options: create_match::CreateMatchOptions::users(&auth_user, &conn),
                    selected: Some(form.player_name_1.to_string()),
                }
            }
            "agent" => create_match::CreateMatchFormSelects {
                i: 1,
                options: create_match::CreateMatchOptions::agents(&auth_user, &conn),
                selected: Some(form.player_name_1.to_string()),
            },
            _ => create_match::CreateMatchFormSelects::default(&auth_user, 1),
        };
        let red_select = match form.player_type_2.as_str() {
            "me" => {
                if form.player_name_2 != auth_user.username {
                    red_error = Some("Me must be you.".to_string());
                }
                create_match::CreateMatchFormSelects {
                    i: 2,
                    options: create_match::CreateMatchOptions::me(&auth_user),
                    selected: Some(auth_user.username.clone()),
                }
            }
            "user" => {
                if form.player_type_2 == auth_user.username {
                    red_error = Some("Select 'me' for yourself.".to_string());
                }
                create_match::CreateMatchFormSelects {
                    i: 2,
                    options: create_match::CreateMatchOptions::users(&auth_user, &conn),
                    selected: Some(form.player_name_2.to_string()),
                }
            }
            "agent" => create_match::CreateMatchFormSelects {
                i: 2,
                options: create_match::CreateMatchOptions::agents(&auth_user, &conn),
                selected: Some(form.player_name_2.to_string()),
            },
            _ => create_match::CreateMatchFormSelects::default(&auth_user, 2),
        };

        match (form.player_type_1.as_str(), form.player_type_2.as_str()) {
            ("user", "user") => {
                // Can't create a game between two users that aren't you.
                blue_error = Some(
                    "You must be one of the players unless the game is all AI agents.".to_string(),
                );
                red_error = Some(
                    "You must be one of the players unless the game is all AI agents.".to_string(),
                );
            }
            ("user", "agent") => {
                // Can't create a game between a user that isn't you and an agent.
                blue_error = Some(
                    "You must be one of the players unless the game is all AI agents.".to_string(),
                );
            }
            ("agent", "user") => {
                // Can't create a game between a user that isn't you and an agent.
                red_error = Some(
                    "You must be one of the players unless the game is all AI agents.".to_string(),
                );
            }
            _ => {}
        }

        if blue_error.is_some() || red_error.is_some() {
            return (
                HeaderMap::new(),
                create_match::CreateMatchForm {
                    blue: blue_select,
                    red: red_select,
                    blue_error,
                    red_error,
                }
                .into_response(),
            );
        }

        let lookup_player = |player_type: &str, player_name: &str| {
            match player_type {
                "me" => Ok((
                    auth_user.id,
                    types::Player::User(types::User {
                        username: auth_user.username.clone(),
                    }),
                )),
                "user" => {
                    if let Some((id, username)) = conn
                        .query_row(
                            r#"
                        SELECT id, username from user WHERE username = ?;
                    "#,
                            [player_name],
                            |row| {
                                let id = row.get(0)?;
                                let username = row.get(1)?;
                                Ok((id, username))
                            },
                        )
                        .optional()
                        .unwrap()
                    {
                        Ok((id, types::Player::User(types::User { username })))
                    } else {
                        Err(format!("User {} not found.", player_name))
                    }
                }
                "agent" => {
                    // TODO: Make sure agent names have same rules as user names.
                    // Can only contain letters, numbers and hyphens and underscores.
                    if let Some((split_username, split_agentname)) = player_name.split_once('/') {
                        if let Some((id, username, agentname)) = conn
                            .query_row(
                                r#"
                                SELECT
                                  agent.id,
                                  user.username,
                                  agent.agentname
                                FROM agent
                                JOIN user ON agent.user_id = user.id
                                WHERE user.username = ?
                                AND agent.agentname = ?
                                AND agent.game = 'connect4'
                            "#,
                                [&split_username, &split_agentname],
                                |row| {
                                    let id = row.get(0)?;
                                    let username = row.get(1)?;
                                    let agentname = row.get(2)?;
                                    Ok((id, username, agentname))
                                },
                            )
                            .optional()
                            .unwrap()
                        {
                            Ok((
                                id,
                                types::Player::Agent(types::Agent {
                                    game: types::Game::Connect4,
                                    username,
                                    agentname,
                                }),
                            ))
                        } else {
                            Err(format!("Agent {} not found.", player_name))
                        }
                    } else {
                        Err(format!("Agent {} not found.", player_name))
                    }
                }
                _ => unreachable!(),
            }
        };

        let (blue_player_id, blue_player, red_player_id, red_player) = {
            let blue_player_result = lookup_player(&form.player_type_1, &form.player_name_1);
            if let Err(e) = &blue_player_result {
                blue_error = Some(e.clone());
            }

            let red_player_result = lookup_player(&form.player_type_2, &form.player_name_2);
            if let Err(e) = &red_player_result {
                red_error = Some(e.clone());
            }

            if blue_error.is_some() || red_error.is_some() {
                return (
                    HeaderMap::new(),
                    create_match::CreateMatchForm {
                        blue: blue_select,
                        red: red_select,
                        blue_error,
                        red_error,
                    }
                    .into_response(),
                );
            }

            let (blue_player_id, blue_player) = blue_player_result.unwrap();
            let (red_player_id, red_player) = red_player_result.unwrap();
            (blue_player_id, blue_player, red_player_id, red_player)
        };

        let (blue_player_user_id, blue_player_agent_id) = match &blue_player {
            types::Player::User(user) => (Some(blue_player_id), None),
            types::Player::Agent(agent) => (None, Some(blue_player_id)),
        };
        let (red_player_user_id, red_player_agent_id) = match &red_player {
            types::Player::User(user) => (Some(red_player_id), None),
            types::Player::Agent(agent) => (None, Some(red_player_id)),
        };

        println!(
            "blue_player_id: {}, blue_player: {:?}",
            blue_player_id, blue_player
        );
        println!(
            "red_player_id: {}, red_player: {:?}",
            red_player_id, red_player
        );

        // New Match
        let game = types::Game::Connect4;
        let players = vec![blue_player, red_player];
        let turns: Vec<types::Turn<types::Connect4Action>> = vec![
            types::Turn {
                number: 0,
                player: None,
                action: None,
            }
        ];
        let turn = 0;
        let status = types::Status::InProgress { next_player: 0 };
        let state = types::Connect4State {
            board: vec![None; 42],
        };

        println!("game: {}", serde_json::to_string(&game).unwrap());

        // todo: handle sqlite errors, hopefully everything is validated by now tho.
        let match_id = {
            let tx = conn.transaction().unwrap();
            tx.execute(
                r#"
                INSERT INTO match (game, created_by)
                VALUES (?, ?)
            "#,
                params!["connect4", auth_user.id],
            )
            .unwrap();
            let match_id = tx.last_insert_rowid();
            tx.execute(
                r#"
                INSERT INTO match_player (match_id, number, user_id, agent_id)
                VALUES (?, ?, ?, ?)
            "#,
                params![
                    // blue player
                    match_id,
                    0,
                    blue_player_user_id,
                    blue_player_agent_id,
                ],
            )
                .unwrap();
            tx.execute(
                r#"
                INSERT INTO match_player (match_id, number, user_id, agent_id)
                VALUES (?, ?, ?, ?)
            "#,
                params![
                    // red player
                    match_id,
                    1,
                    red_player_user_id,
                    red_player_agent_id
                ],
            ).unwrap();
            tx.execute(
                r#"
                INSERT INTO match_turn (match_id, number, player, action, status, winner, next_player, state)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
                params![
                    match_id,
                    turns[0].number,
                    turns[0].player,
                    None::<String>,
                    "in_progress",
                    None::<usize>,
                    Some(0),
                    serde_json::to_string(&state).unwrap()
                    ],
            )
            .unwrap();

            tx.commit().unwrap();
            match_id
        };

        println!("match_id: {}", match_id);

        let form = create_match::CreateMatchForm::default(&auth_user);
        let location =
            json!({"path": format!("/app/games/connect4/matches/{match_id}"), "target": "#main"});

        let mut headers = HeaderMap::new();
        headers.insert("hx-location", location.to_string().parse().unwrap());
        (headers, form.into_response())
    }).instrument(info_span!("create_match"))
    .await
    .unwrap()
}

#[tracing::instrument(skip(state))]
pub async fn connect4_selects<'a>(
    auth_user: types::UserRecord,
    State(state): State<Arc<AppState>>,
    query: Query<create_match::CreateMatchSelectsQuery>,
) -> impl IntoResponse {
    tokio::task::spawn_blocking(move || {
        let conn = state.pool.get().unwrap();

        match query.fetch(&auth_user, &conn) {
            Ok(selects) => (StatusCode::OK, selects.into_response()),
            Err(error) => (
                StatusCode::BAD_REQUEST,
                askama_axum::IntoResponse::into_response(error),
            ),
        }
    })
    .await
    .unwrap()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn connect4_match<'a>(
    _auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut m = types::Match {
        id: 123,
        game: types::Game::Connect4,
        players: vec![
            types::Player::User(types::User {
                username: "user1".to_string(),
            }),
            types::Player::User(types::User {
                username: "steve".to_string(),
            }),
        ],
        turns: vec![
            types::Turn {
                number: 0,
                player: None,
                action: None,
            },
            types::Turn {
                number: 1,
                player: Some(0),
                action: Some(types::Connect4Action { column: 0 }),
            },
        ],
        turn: 1,
        status: types::Status::InProgress { next_player: 1 },
        state: types::Connect4State {
            board: vec![None; 42],
        },
    };

    m.state.board[0] = Some(0);
    m.state.board[1] = Some(1);

    let template = templates::Connect4Match {
        _layout: app_layout,
        connect4_match: m,
    };
    template.into_response()
}

#[derive(Deserialize, Debug)]
pub struct CreateTurnFormData {
    pub player: usize,
    pub column: usize,
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn connect4_match_create_turn<'a>(
    _auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
    Form(form): Form<CreateTurnFormData>,
) -> impl IntoResponse {
    println!("{:?}", form);

    let mut m = types::Match {
        id: 123,
        game: types::Game::Connect4,
        players: vec![
            types::Player::User(types::User {
                username: "user1".to_string(),
            }),
            types::Player::User(types::User {
                username: "steve".to_string(),
            }),
        ],
        turns: vec![
            types::Turn {
                number: 0,
                player: None,
                action: None,
            },
            types::Turn {
                number: 1,
                player: Some(0),
                action: Some(types::Connect4Action { column: 0 }),
            },
        ],
        turn: 1,
        status: types::Status::Over { winner: None },
        state: types::Connect4State {
            board: vec![None; 42],
        },
    };

    m.state.board[0] = Some(0);
    m.state.board[1] = Some(1);

    let template = templates::Connect4Match {
        _layout: app_layout,
        connect4_match: m,
    };
    template.into_response()
}
