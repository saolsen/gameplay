use axum::{
    routing::post,
    Router,
    response::Json,
};
use rand::Rng;
use serde_json::{Value, json};

// This is an example of a "from scratch" agent.
// You can write an agent in any language as long
// as it can host an http server and talk json.
async fn agent(Json(game_state): Json<Value>) -> Json<Value> {
    let board_json = game_state.get("board").unwrap().as_array().unwrap();
    let board: Vec<Option<u64>> = board_json.iter().map(|v| {
        if v.is_null() {
            None
        } else {
            Some(v.as_u64().unwrap())
        }
    }).collect();
    let _next_player = game_state.get("next_player").unwrap().as_u64().unwrap();

    let mut avail_columns: Vec<usize> = vec![];
    for i in 0..7 {
        if board[i * 6 + 5].is_none() {
            avail_columns.push(i);
        }
    }

    let mut rng = rand::thread_rng();
    let i = rng.gen_range(0..avail_columns.len());
    let column = avail_columns[i];

    Json(json!({ "column": column }))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(agent));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
