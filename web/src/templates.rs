use askama::Template;
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::ops::Deref;
use std::sync::Arc;

use crate::config;
use crate::types;
use crate::web;

#[derive(Template)]
#[template(path = "refresh.html")]
pub struct Refresh<'a> {
    pub clerk_pub_api_key: &'a str,
}

#[derive(Template)]
#[template(path = "web_layout.html")]
pub struct WebLayout<'a> {
    pub clerk_pub_api_key: &'a str,
    pub main_only: bool,
}

#[async_trait]
impl<'a, S> FromRequestParts<S> for WebLayout<'a>
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut main_only = false;
        if let Some(target) = parts.headers.get("hx-target") {
            if target == "main" {
                main_only = true;
            }
        }

        return Ok(WebLayout {
            clerk_pub_api_key: &config::CLERK_PUB_API_KEY,
            main_only,
        });
    }
}

#[derive(Template)]
#[template(path = "app_layout.html")]
pub struct AppLayout<'a> {
    pub clerk_pub_api_key: &'a str,
    // The user making the request. All app routes require authentication
    pub auth_user: types::UserRecord,
    // True if this is an htmx request that is replacing #main
    // This lets the template know to only render the #main div
    pub main_only: bool,
}

#[async_trait]
impl<'a> FromRequestParts<Arc<web::AppState>> for AppLayout<'a> {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<web::AppState>,
    ) -> Result<Self, Self::Rejection> {
        // todo: cache this so we don't do it twice
        let auth_user = match types::UserRecord::from_request_parts(parts, state).await {
            Ok(auth_user) => auth_user,
            Err(rejection) => {
                return Err(rejection.into_response());
            }
        };

        let mut main_only = false;
        if let Some(target) = parts.headers.get("hx-target") {
            if target == "main" {
                main_only = true;
            }
        }

        return Ok(AppLayout {
            clerk_pub_api_key: &config::CLERK_PUB_API_KEY,
            auth_user,
            main_only,
        });
    }
}

#[derive(Template)]
#[template(path = "app_index.html")]
pub struct AppIndex<'a> {
    pub _layout: &'a AppLayout<'a>,
}

impl<'a> Deref for AppIndex<'a> {
    type Target = AppLayout<'a>;

    fn deref(&self) -> &Self::Target {
        self._layout
    }
}
