use askama::Template;
use serde::Deserialize;

use crate::types;

pub enum CreateMatchOptions {
    Me(String),
    User(Vec<String>),
    Agent(Vec<String>),
}

// This maybe goes somewhere else. It's more of a service function, but also has to be called here.
impl CreateMatchOptions {
    pub fn me(auth_user: &types::UserRecord) -> Self {
        Self::Me(auth_user.username.to_string())
    }
    pub fn users(_auth_user: &types::UserRecord, _conn: &types::Conn) -> Self {
        // todo: query db for users
        Self::User(vec!["gabe".to_string(), "steve".to_string()])
    }
    pub fn agents(_auth_user: &types::UserRecord, _conn: &types::Conn) -> Self {
        // todo: query db for agents
        Self::Agent(vec!["steve/mcts".to_string()])
    }
}

#[derive(Template)]
#[template(path = "create_match_form_selects.html")]
pub struct CreateMatchFormSelects {
    pub i: usize,
    pub options: CreateMatchOptions,
    pub selected: Option<String>,
}

impl CreateMatchFormSelects {
    pub fn default(auth_user: &types::UserRecord, i: usize) -> Self {
        Self {
            i,
            options: CreateMatchOptions::me(auth_user),
            selected: None,
        }
    }
}

// query the type to get the options for the select.
// Happens whenever the type changes.
#[derive(Deserialize, Debug)]
pub struct CreateMatchSelectsQuery {
    pub player_type_1: Option<String>,
    pub player_type_2: Option<String>,
}

impl CreateMatchSelectsQuery {
    pub fn fetch(
        &self,
        auth_user: &types::UserRecord,
        conn: &types::Conn,
    ) -> Result<CreateMatchFormSelects, String> {
        let (player_type, n) = match (&self.player_type_1, &self.player_type_2) {
            (Some(player_type_1), None) => {
                let n = 1;
                (player_type_1, n)
            }
            (None, Some(player_type_2)) => {
                let n = 2;
                (player_type_2, n)
            }
            _ => {
                return Err("Invalid query params".to_owned());
            }
        };

        let options = match player_type.as_str() {
            "me" => CreateMatchOptions::me(auth_user),
            "user" => CreateMatchOptions::users(auth_user, conn),
            "agent" => CreateMatchOptions::agents(auth_user, conn),
            _ => {
                return Err("Invalid query params".to_owned());
            }
        };

        let selects = CreateMatchFormSelects {
            i: n,
            options,
            selected: None,
        };

        Ok(selects)
    }
}

#[derive(Template)]
#[template(path = "create_match_form.html")]
pub struct CreateMatchForm {
    pub blue: CreateMatchFormSelects,
    pub red: CreateMatchFormSelects,
    pub blue_error: Option<String>,
    pub red_error: Option<String>,
}

impl CreateMatchForm {
    pub fn default(auth_user: &types::UserRecord) -> Self {
        Self {
            blue: CreateMatchFormSelects::default(auth_user, 1),
            red: CreateMatchFormSelects::default(auth_user, 2),
            blue_error: None,
            red_error: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateMatchFormData {
    pub player_type_1: String,
    pub player_name_1: String,
    pub player_type_2: String,
    pub player_name_2: String,
}
