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
    _auth_user: types::UserRecord,
    app_layout: templates::AppLayout<'a>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let hello = templates::AppIndex {
        _layout: app_layout,
    };
    hello.into_response()
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
    // todo: on error form should have same things selected as before
    let form = templates::CreateMatchForm { auth_user };
    let location =
        json!({"path": format!("/app/games/connect4/matches/{}", 123), "target": "#main"});
    (
        [("hx-location", location.to_string())],
        form.into_response(),
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
    let (player_type, n, player) = match (&query.player_type_1, &query.player_type_2) {
        (Some(player_type_1), None) => {
            let n = 1;
            let player = "blue";
            (player_type_1, n, player)
        }
        (None, Some(player_type_2)) => {
            let n = 2;
            let player = "red";
            (player_type_2, n, player)
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("Invalid query params".to_owned()),
            );
        }
    };

    let selects = match player_type.as_str() {
        "me" => {
            format!(
                r#"<input name="player_name_1" type="hidden" value="{}">"#,
                auth_user.username
            )
        }
        "user" => {
            let options = vec![
                format!(r#"<option value="{}">{}</option>"#, "steveo", "steveo"),
                format!(r#"<option value="{}">{}</option>"#, "gabe", "gabe"),
            ];

            format!(
                r#"
                <label for="{}_player" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">username</label>
                <select name="player_name_{}" id="{}_player" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {}
                </select>
                "#,
                player,
                n,
                player,
                options.join("\n")
            )
        }
        "agent" => {
            let options = vec![
                format!(
                    r#"<option value="{}/{}">{}/{}</option>"#,
                    "steveo", "random", "steveo", "random"
                ),
                format!(
                    r#"<option value="{}/{}">{}/{}</option>"#,
                    "gabe", "smart", "gabe", "smart"
                ),
            ];

            format!(
                r#"
                <label for="{}_player" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">agentname</label>
                <select name="player_name_{}" id="{}_player" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {}
                </select>
                "#,
                player,
                n,
                player,
                options.join("\n")
            )
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Html("Invalid player type".to_owned()),
            );
        }
    };

    (StatusCode::OK, Html(selects))
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
