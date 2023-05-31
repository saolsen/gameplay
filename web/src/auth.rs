use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Redirect;
use cookie::Cookie;
use jwt_simple::prelude::*;
use lazy_static::lazy_static;
use rusqlite::OptionalExtension;
use std::sync::Arc;

use crate::config;
use crate::types;
use crate::web;

lazy_static! {
    static ref KEY: RS256PublicKey =
        RS256PublicKey::from_pem(&config::CLERK_PUB_ENCRYPTION_KEY).unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct ClerkClaims {
    clerk_id: String,
    username: String,
    first_name: String,
    last_name: String,
    email: String,
}

#[async_trait]
impl FromRequestParts<Arc<web::AppState>> for types::UserRecord {
    type Rejection = Redirect;

    #[tracing::instrument(skip(parts, state))]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<web::AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Get clerk session cookie.
        if let Some(header) = parts.headers.get("cookie") {
            for cookie in Cookie::split_parse(header.to_str().unwrap()) {
                let c = cookie.unwrap();
                if c.name() == "__session" {
                    // Validate the token
                    let s = c.value();
                    if let Ok(claims) = KEY.verify_token::<ClerkClaims>(s, None) {
                        // Look up the user.
                        let state = state.clone();
                        let clerk_id = claims.custom.clerk_id.clone();
                        let username = claims.custom.username.clone();
                        let first_name = claims.custom.first_name.clone();
                        let last_name = claims.custom.last_name.clone();
                        let email = claims.custom.email.clone();
                        let user = tokio::task::spawn_blocking(move || {
                            let conn = state.pool.get().unwrap();
                            let user_id = {
                                // Check if user data is up to date.
                                let user_id = conn.query_row(
                                    r#"
                                        SELECT id
                                        FROM user
                                        WHERE clerk_id = ?1
                                        AND username = ?2
                                        AND first_name = ?3
                                        AND last_name = ?4
                                        AND email = ?5
                                    "#, [
                                    &clerk_id,
                                    &username,
                                    &first_name,
                                    &last_name,
                                    &email], |row| {
                                    row.get(0)
                                }).optional().unwrap();
                                if let Some(id) = user_id {
                                    id
                                } else {
                                    // Otherwise insert or update the user.
                                    conn.execute(
                                        r#"
                                            INSERT INTO user (clerk_id, username, first_name, last_name, email)
                                            VALUES (?1, ?2, ?3, ?4, ?5)
                                            ON CONFLICT(clerk_id) DO UPDATE SET
                                                username = excluded.username,
                                                first_name = excluded.first_name,
                                                last_name = excluded.last_name,
                                                email = excluded.email
                                        "#,[
                                        &clerk_id,
                                        &username,
                                        &first_name,
                                        &last_name,
                                        &email]
                                    ).unwrap();
                                    conn.last_insert_rowid()
                                }
                            };
                            types::UserRecord {
                                id: user_id,
                                clerk_id: claims.custom.clerk_id,
                                username: claims.custom.username,
                                first_name: claims.custom.first_name,
                                last_name: claims.custom.last_name,
                                email: claims.custom.email,
                            }
                        })
                            .await
                            .unwrap();

                        return Ok(user);
                    }
                }
            }
        }
        // todo: @fragile if it was a post or something with query params it would break.
        Err(Redirect::temporary(&format!(
            "/refresh?next={}",
            parts.uri.path()
        )))
    }
}
