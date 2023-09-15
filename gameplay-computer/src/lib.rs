use std::env;
use sqlx::{Connection, PgConnection};

pub async fn foo() -> String {
    let mut conn = PgConnection::connect(env::var("DATABASE_URL").unwrap().as_str()).await.unwrap();
    let row: (i64,) = sqlx::query_as("SELECT $1").bind(150_i64).fetch_one(&mut conn).await.unwrap();
    println!("The number is: {}", row.0);

    "Hello, world!".to_string()
}