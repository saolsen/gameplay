use crate::{config, migrations, templates, types};
use askama_axum::IntoResponse as _;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
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
        _layout: &app_layout,
    };
    hello.into_response()
}

