use crate::connect4::Connect4Check;
use crate::forms::{create_agent, create_match};
use crate::{config, connect4, matches, migrations, templates, types};
use askama_axum::IntoResponse as _;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
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
    .instrument(info_span!("get_version"))
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
        create_agent: create_agent::CreateAgentForm::default(),
    };
    index.into_response()
}

#[tracing::instrument(skip(state))]
pub async fn create_agent<'a>(
    auth_user: types::UserRecord,
    State(state): State<Arc<AppState>>,
    Form(form): Form<create_agent::CreateAgentFormData>,
) -> impl IntoResponse {
    let (agent_id, response) = tokio::task::spawn_blocking(move || {
        let mut conn = state.pool.get().unwrap();
        assert_eq!(form.game, "connect4");

        let mut agentname_error = None;
        let mut url_error = None;

        let agentname = form.agentname;
        let url = form.url;

        let valid_agentname = agentname
            .chars()
            .all(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_'));
        if !valid_agentname {
            agentname_error = Some(
                "Agent name can only contain letters, numbers, hyphens and underscores".to_owned(),
            );
        } else {
            let name_exists = conn
                .query_row(
                    r#"
                        select 1
                        from agent
                        where user_id = ? and agentname = ?
                    "#,
                    params![auth_user.id, agentname],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .unwrap()
                .unwrap_or(0)
                != 0;
            if name_exists {
                agentname_error = Some("You already have an agent with that name".to_owned());
            }
        }

        let uri = url.parse::<Uri>();
        match &uri {
            Err(e) => {
                url_error = Some("URL is not valid".to_owned());
            }
            Ok(uri) => {
                if uri.scheme_str() != Some("http") && uri.scheme_str() != Some("https") {
                    url_error = Some("URL must be http or https".to_owned());
                }
                if uri.host().is_none() {
                    url_error = Some("URL must have a host".to_owned());
                }
            }
        }

        if agentname_error.is_some() || url_error.is_some() {
            return (
                None,
                (
                    HeaderMap::new(),
                    create_agent::CreateAgentForm {
                        game: "connect4".to_owned(),
                        agentname,
                        url,
                        agentname_error,
                        url_error,
                    }
                    .into_response(),
                ),
            );
        }

        let url = uri.unwrap().to_string();

        // Create the agent.
        let agent_id = {
            let tx = conn.transaction().unwrap();
            tx.execute(
                r#"
                    insert into agent (user_id, game, agentname)
                    values (?, ?, ?)
                "#,
                params![auth_user.id, "connect4", agentname],
            )
            .unwrap();
            let agent_id = tx.last_insert_rowid();
            tx.execute(
                r#"
                    insert into agent_http (agent_id, url, status, error)
                    values (?, ?, 'pending', null)
                "#,
                params![agent_id, url],
            )
            .unwrap();
            tx.commit().unwrap();
            agent_id
        };

        let form = create_agent::CreateAgentForm::default();
        let mut headers = HeaderMap::new();
        headers.insert("hx-trigger", "AgentUpdate".parse().unwrap());
        (Some(agent_id), (headers, form.into_response()))
    })
    .instrument(info_span!("create_agent_sync"))
    .await
    .unwrap();

    // todo: Schedule the task that validates the agent
    if let Some(agent_id) = agent_id {
        eprintln!("agent_id: {}", agent_id);
    }

    response
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
            types::Player::User(_) => (Some(blue_player_id), None),
            types::Player::Agent(_) => (None, Some(blue_player_id)),
        };
        let (red_player_user_id, red_player_agent_id) = match &red_player {
            types::Player::User(_) => (Some(red_player_id), None),
            types::Player::Agent(_) => (None, Some(red_player_id)),
        };

        // New Match
        let game = types::Game::Connect4;
        let turns: Vec<types::Turn<types::Connect4Action>> = vec![
            types::Turn {
                number: 0,
                player: None,
                action: None,
            }
        ];
        let state = types::Connect4State {
            board: vec![None; 42],
        };

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

        let form = create_match::CreateMatchForm::default(&auth_user);
        let location =
            json!({"path": format!("/app/games/connect4/matches/{match_id}"), "target": "#main"});

        let mut headers = HeaderMap::new();
        headers.insert("hx-location", location.to_string().parse().unwrap());
        (headers, form.into_response())
    }).instrument(info_span!("create_match_sync"))
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
    .instrument(info_span!("get_selects_sync"))
    .await
    .unwrap()
}

#[tracing::instrument(skip(app_layout, state))]
pub async fn connect4_match<'a>(
    _auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<i64>,
) -> impl IntoResponse {
    // Todo: Authorization
    let maybe_match: Option<types::Match<types::Connect4Action, types::Connect4State>> =
        tokio::task::spawn_blocking(move || {
            let conn = state.pool.get().unwrap();
            matches::get_by_id(&conn, match_id)
        })
        .instrument(info_span!("get_match_sync"))
        .await
        .unwrap();

    if let Some(m) = maybe_match {
        let template = templates::Connect4Match {
            _layout: app_layout,
            connect4_match: m,
        };
        (StatusCode::OK, template.into_response())
    } else {
        (
            StatusCode::NOT_FOUND,
            askama_axum::IntoResponse::into_response("Match not found"),
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateTurnFormData {
    pub player: usize,
    pub column: usize,
}

pub async fn connect4_match_create_turn<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<i64>,
    Form(form): Form<CreateTurnFormData>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let conn = state.pool.get().unwrap();
        let maybe_match = matches::get_by_id(&conn, match_id);

        let match_ = match maybe_match {
            None => return Err((StatusCode::BAD_REQUEST, "Match not found".to_owned())),
            Some(match_) => match_,
        };

        // Validate the turn.
        match &match_.status {
            types::Status::Over { .. } => {
                return Err((StatusCode::BAD_REQUEST, "Match is over".to_owned()))
            }
            types::Status::InProgress { next_player } => {
                if *next_player != form.player {
                    return Err((StatusCode::BAD_REQUEST, "Not your turn".to_owned()));
                }

                let player = &match_.players[*next_player];
                match player {
                    types::Player::User(user) => {
                        if user.username != auth_user.username {
                            return Err((StatusCode::BAD_REQUEST, "Not your turn".to_owned()));
                        }
                    }
                    types::Player::Agent(_) => {
                        return Err((StatusCode::BAD_REQUEST, "Not your turn".to_owned()));
                    }
                }
            }
        };

        // Run the logic for the turn.
        let action = types::Connect4Action {
            column: form.column,
        };
        let mut state = match_.state;
        if let Err(error) = connect4::apply_action(&mut state, &action, form.player) {
            return Err((StatusCode::BAD_REQUEST, error.to_string()));
        }

        // Check for win
        let check = connect4::check(&state);
        let (status, winner, next_player) = match check {
            Connect4Check::Winner(player) => {
                ("over", Some(player), None)
            },
            Connect4Check::Tie => {
                ("over", None, None)
            },
            Connect4Check::InProgress => {
                ("in_progress", None, Some((form.player + 1) % 2))
            },
        };

        // Insert the turn, if the turn number already exists that means the turn was already taken.
        let turn_number = match_.turn + 1;
        let insert_result = conn.execute(
            r#"
                INSERT INTO match_turn (match_id, number, player, action, status, winner, next_player, state)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                    match_id,
                    turn_number,
                    form.player,
                    serde_json::to_string(&action).unwrap(),
                    status,
                    winner,
                    next_player,
                    serde_json::to_string(&state).unwrap()
                    ],
        );
        if let Err(rusqlite::Error::SqliteFailure(error, _)) = insert_result {
            if error.code == rusqlite::ErrorCode::ConstraintViolation {
                return Err((StatusCode::BAD_REQUEST, "Turn already taken".to_owned()));
            } else {
                panic!("Unexpected error: {:?}", error)
            }
        }

        if let Some(_match) = matches::get_by_id(&conn, match_id) {
            Ok(_match)
        } else {
            Err((
                StatusCode::NOT_FOUND,
                "Match not found".to_owned(),
            ))
        }
    })
    .instrument(info_span!("create_turn_sync"))
    .await
    .unwrap();

    match result {
        Ok(match_) => {
            let template = templates::Connect4Match {
                _layout: app_layout,
                connect4_match: match_,
            };
            (StatusCode::OK, template.into_response())
        }
        Err((status, body)) => (status, askama_axum::IntoResponse::into_response(body)),
    }
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_me<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "me".to_owned(),
    };
    index.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_me_matches<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "me matches".to_owned(),
    };
    index.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_me_agents<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "me agents".to_owned(),
    };
    index.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_games<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "games".to_owned(),
    };
    index.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_users<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "users".to_owned(),
    };
    index.into_response()
}

#[tracing::instrument(skip(app_layout, _state))]
pub async fn app_agents<'a>(
    auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let index = templates::AppPlayground {
        _layout: app_layout,
        page: "agents".to_owned(),
    };
    index.into_response()
}
