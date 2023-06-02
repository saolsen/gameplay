use crate::{config, migrations, templates, types};
use askama_axum::IntoResponse as _;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Form;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use crate::templates::{CreateMatchFormSelects, CreateMatchOptions};

#[derive(Debug)]
pub struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
}

impl AppState {
    pub fn new() -> Self {
        let manager = SqliteConnectionManager::file(&*config::DB).with_init(|c| {
            //let manager = SqliteConnectionManager::memory().with_init(|c| {
            c.execute_batch(
                r#"
                    PRAGMA journal_mode = wal;
                    PRAGMA synchronous = normal;
                    PRAGMA foreign_keys = on;
                "#,
            )
        });
        let pool = Pool::new(manager).unwrap();
        {
            let mut conn = pool.get().unwrap();
            migrations::migrate(&mut conn).unwrap();
        }
        Self { pool }
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
    let _result = tokio::task::spawn_blocking(move || {
        let conn = state.pool.get().unwrap();
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
        create_match: templates::CreateMatchForm {
            blue: templates::CreateMatchFormSelects {
                i: 1,
                options: templates::CreateMatchOptions::Me(auth_user.username.clone()),
                selected: Some(auth_user.username.clone())
            },
            red: templates::CreateMatchFormSelects {
                i: 2,
                options: templates::CreateMatchOptions::User(vec!["gabe".to_string(), "steve".to_string()]),
                selected: Some("steve".to_owned()),
            }
        },
    };
    index.into_response()
}

#[derive(Deserialize, Debug)]
pub struct CreateMatchFormData {
    pub player_type_1: String,
    pub player_name_1: String,
    pub player_type_2: String,
    pub player_name_2: String,
}

#[tracing::instrument(skip(_state))]
pub async fn connect4_create_match<'a>(
    auth_user: types::UserRecord,
    State(_state): State<Arc<AppState>>,
    Form(form): Form<CreateMatchFormData>,
) -> impl IntoResponse {
    // two users that are not you is not allowed
    // make sure user isn't you, that's hackers

    //todo: can write this now
    // STEVE THIS NEXT

    match form.player_type_1.as_str() {
        "me" => {
            println!("me")
        }
        "user" => {
            println!("user")
        }
        "agent" => {
            println!("agent")
        }
        _ => {
            println!("wat: {}", form.player_type_1)
        }
    }


    // todo: on error, form should have same things selected as before
    // let form = templates::CreateMatchForm { auth_user,
    // blue:
    //
    // };
    let location =
        json!({"path": format!("/app/games/connect4/matches/{}", 123), "target": "#main"});
    (
        [("hx-location", location.to_string())],
        "todo",
    )
}

#[derive(Deserialize, Debug)]
pub struct SelectsQuery {
    pub player_type_1: Option<String>,
    pub player_type_2: Option<String>,
}

#[tracing::instrument(skip(_state))]
pub async fn connect4_selects<'a>(
    auth_user: types::UserRecord,
    State(_state): State<Arc<AppState>>,
    query: Query<SelectsQuery>,
) -> impl IntoResponse {
    let (player_type, n) = match (&query.player_type_1, &query.player_type_2) {
        (Some(player_type_1), None) => {
            let n = 1;
            (player_type_1, n)
        }
        (None, Some(player_type_2)) => {
            let n = 2;
            (player_type_2, n)
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("Invalid query params".to_owned()),
            );
        }
    };

    let options = match player_type.as_str() {
        "me" => {
            CreateMatchOptions::Me(auth_user.username.clone())
        },
        "user" => {
            CreateMatchOptions::User(vec!["gabe".to_string(), "steve".to_string()])
        },
        "agent" => {
            CreateMatchOptions::Agent(vec!["random".to_string(), "minimax".to_string()])
        },
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("Invalid query params".to_owned()),
            );
        }
    };

    let selects = CreateMatchFormSelects{
        i: n,
        options,
        selected: None,
    };

    (StatusCode::OK, Html(selects.to_string()))
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
