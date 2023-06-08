use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Redirect;
use cookie::Cookie;
use jwt_simple::prelude::*;
use rusqlite::OptionalExtension;
use std::sync::Arc;
use tracing::Instrument;

use crate::types;
use crate::web;

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
                    let s = c.value().to_owned();
                    let state = state.clone();
                    let user = tokio::task::spawn_blocking(move || {
                        if let Ok(claims) = state.key.verify_token::<ClerkClaims>(&s, None) {
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
                                        &claims.custom.clerk_id,
                                        &claims.custom.username,
                                        &claims.custom.first_name,
                                        &claims.custom.last_name,
                                        &claims.custom.email], |row| {
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
                                            &claims.custom.clerk_id,
                                            &claims.custom.username,
                                            &claims.custom.first_name,
                                            &claims.custom.last_name,
                                            &claims.custom.email]
                                    ).unwrap();
                                    // Get again instead of last_row_id because it could be
                                    // an update.
                                    conn.query_row(
                                        r#"
                                        SELECT id
                                        FROM user
                                        WHERE clerk_id = ?1
                                        AND username = ?2
                                        AND first_name = ?3
                                        AND last_name = ?4
                                        AND email = ?5
                                    "#, [
                                            &claims.custom.clerk_id,
                                            &claims.custom.username,
                                            &claims.custom.first_name,
                                            &claims.custom.last_name,
                                            &claims.custom.email], |row| {
                                            row.get(0)
                                        }).unwrap()
                                }
                            };
                            return Some(types::UserRecord {
                                id: user_id,
                                clerk_id: claims.custom.clerk_id,
                                username: claims.custom.username,
                                first_name: claims.custom.first_name,
                                last_name: claims.custom.last_name,
                                email: claims.custom.email,
                            })
                        }
                        None
                    })
                    .instrument(tracing::info_span!("validate jwt"))
                    .await
                    .unwrap();

                    if let Some(user) = user {
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
