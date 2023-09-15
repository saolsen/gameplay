use serde_json::json;
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use gameplay_computer::foo;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    eprintln!("Does this actually log stuff?");
    println!("Seems like it!");

    _ = foo().await;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": "Gameplay Stuff"
            })
                .to_string()
                .into(),
        )?)
}
