use crate::forms::create_match;
use crate::{config, migrations, templates, types};
use askama_axum::IntoResponse as _;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Form;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState {
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

#[tracing::instrument(skip(state))]
pub async fn connect4_create_match<'a>(
    auth_user: types::UserRecord,
    State(state): State<Arc<AppState>>,
    Form(form): Form<create_match::CreateMatchFormData>,
) -> impl IntoResponse {
    // todo: Validate the player names and agent names as being real.
    // these will be text box auto complete fields eventually, not selects so
    // they need to check their inputs.
    
    tokio::task::spawn_blocking(move || {
        let conn = state.pool.get().unwrap();
        if let Err(err_form) = form.validate(&auth_user, &conn) {
            return (HeaderMap::new(), err_form.into_response());
        }

        // todo: create the match

        let form = create_match::CreateMatchForm::default(&auth_user);
        let location =
            json!({"path": format!("/app/games/connect4/matches/{}", 123), "target": "#main"});

        let mut headers = HeaderMap::new();
        headers.insert("hx-location", location.to_string().parse().unwrap());
        (headers, form.into_response())
    })
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
