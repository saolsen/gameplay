use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing::Span;
use tracing_subscriber::layer::SubscriberExt;

mod auth;
mod config;
mod migrations;
mod templates;
mod types;
mod web;

#[tokio::main]
async fn main() {
    config::load();

    let env = match config::SENTRY_ENV.as_str() {
        "production" => "production".into(),
        "development" => "development".into(),
        _ => panic!("Invalid SENTRY_ENV"),
    };

    // Set up sentry
    std::env::set_var("RUST_BACKTRACE", "1");
    let _guard = sentry::init((
        config::SENTRY_DSN.to_owned(),
        sentry::ClientOptions {
            release: sentry::release_name!(),
            traces_sample_rate: 1.0,
            environment: Some(env),
            ..Default::default()
        },
    ));

    // Set up tracing
    let local_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false);

    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(LevelFilter::INFO)
        .with(sentry_tracing::layer())
        .with(local_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");

    let state = Arc::new(web::AppState::new());

    // todo: Router should go in web.rs
    let app = Router::new()
        .route(
            "/style.css",
            get(|| async {
                (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/css")],
                    web::CSS,
                )
            }),
        )
        .route("/health", get(web::health))
        .route("/refresh", get(web::refresh))
        .route("/", get(web::root))
        .route("/app", get(web::app))
        .route(
            "/app/games/connect4/matches/create_match",
            post(web::connect4_create_match),
        )
        .route(
            "/app/games/connect4/matches/create_match/selects",
            get(web::connect4_selects),
        )
        .route(
            "/app/games/connect4/matches/:match_id",
            get(web::connect4_match),
        )
        .route(
            "/app/games/connect4/matches/:match_id/turns/create_turn",
            post(web::connect4_match_create_turn),
        )
        .with_state(state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    if request.uri().path() == "/health" {
                        return tracing::debug_span!("health-check");
                    }
                    let description = format!("{} {}", request.method(), request.uri().path());

                    tracing::info_span!("http-request",
                        status = tracing::field::Empty,
                        description = %description,
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        headers = ?request.headers(),)
                })
                .on_response(
                    |response: &Response<Body>, _latency: Duration, span: &Span| {
                        span.record(
                            "status",
                            &tracing::field::display(response.status().as_u16()),
                        );
                    },
                ),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
