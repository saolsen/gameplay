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
    pub htmx_target: String,
}

#[async_trait]
impl<'a, S> FromRequestParts<S> for WebLayout<'a>
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let mut htmx_target = "".to_owned();
        if let Some(target) = parts.headers.get("hx-target") {
            htmx_target = target.to_str().unwrap().to_owned();
        }

        return Ok(WebLayout {
            clerk_pub_api_key: &config::CLERK_PUB_API_KEY,
            htmx_target,
        });
    }
}

#[derive(Template)]
#[template(path = "app_layout.html")]
pub struct AppLayout<'a> {
    pub clerk_pub_api_key: &'a str,
    // The user making the request. All app routes require authentication
    pub auth_user: types::UserRecord,
    pub htmx_target: String,
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

        let mut htmx_target = "".to_owned();
        if let Some(target) = parts.headers.get("hx-target") {
            htmx_target = target.to_str().unwrap().to_owned();
        }

        return Ok(AppLayout {
            clerk_pub_api_key: &config::CLERK_PUB_API_KEY,
            auth_user,
            htmx_target,
        });
    }
}

pub enum CreateMatchOptions {
    Me(String),
    User(Vec<String>),
    Agent(Vec<String>),
}

#[derive(Template)]
#[template(path = "create_match_form_selects.html")]
pub struct CreateMatchFormSelects {
    pub i: usize,
    pub options: CreateMatchOptions,
    pub selected: Option<String>,
}

#[derive(Template)]
#[template(path = "create_match_form.html")]
pub struct CreateMatchForm {
    pub blue: CreateMatchFormSelects,
    pub red: CreateMatchFormSelects,
}

#[derive(Template)]
#[template(path = "app_index.html")]
pub struct AppIndex<'a> {
    pub _layout: AppLayout<'a>,
    pub create_match: CreateMatchForm,
}

impl<'a> Deref for AppIndex<'a> {
    type Target = AppLayout<'a>;

    fn deref(&self) -> &Self::Target {
        &self._layout
    }
}

#[derive(Template)]
#[template(path = "connect4_match.html")]
pub struct Connect4Match<'a> {
    pub _layout: AppLayout<'a>,
    pub connect4_match: types::Match<types::Connect4Action, types::Connect4State>,
}

impl<'a> Deref for Connect4Match<'a> {
    type Target = AppLayout<'a>;

    fn deref(&self) -> &Self::Target {
        &self._layout
    }
}