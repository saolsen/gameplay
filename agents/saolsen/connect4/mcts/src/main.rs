use std::env;

use axum::{response::Json, routing::post, Router};

// This agent uses the gameplay crate types. This is currently only
// easy for rust but we can add more helper libs for other languages.
use gameplay::games::connect4::{Action, Connect4};

mod mcts;

async fn agent(Json(game_state): Json<Connect4>) -> Json<Action> {
    let action = mcts::agent(&game_state);
    Json(action)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(agent));

    let port = env::var("PORT").unwrap_or("8000".to_string());

    println!("Listening on {}", port);

    axum::Server::bind(&format!("0.0.0.0:{}", port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
