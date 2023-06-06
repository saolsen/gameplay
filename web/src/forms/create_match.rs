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
        Self::Agent(vec!["steve/random".to_string(), "gabe/mcts".to_string()])
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
    pub errors: Vec<(usize, String)>,
}

impl CreateMatchForm {
    pub fn default(auth_user: &types::UserRecord) -> Self {
        Self {
            blue: CreateMatchFormSelects::default(auth_user, 1),
            red: CreateMatchFormSelects::default(auth_user, 2),
            errors: vec![],
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

impl CreateMatchFormData {
    // Returns a form with errors filled in.
    pub fn validate(
        &self,
        auth_user: &types::UserRecord,
        conn: &types::Conn,
    ) -> Result<(), CreateMatchForm> {
        let mut errors = vec![];
        let blue_select = match self.player_type_1.as_str() {
            "me" => {
                if self.player_name_1 != auth_user.username {
                    errors.push((1, format!("Me must be you.")));
                }
                CreateMatchFormSelects {
                    i: 1,
                    options: CreateMatchOptions::me(auth_user),
                    selected: Some(auth_user.username.clone()),
                }
            }
            "user" => {
                if self.player_type_1 == auth_user.username {
                    errors.push((1, format!("Select 'me' for yourself.")));
                }
                CreateMatchFormSelects {
                    i: 1,
                    options: CreateMatchOptions::users(auth_user, conn),
                    selected: Some(self.player_name_1.to_string()),
                }
            }
            "agent" => CreateMatchFormSelects {
                i: 1,
                options: CreateMatchOptions::agents(auth_user, conn),
                selected: Some(self.player_name_1.to_string()),
            },
            _ => CreateMatchFormSelects::default(auth_user, 1),
        };
        let red_select = match self.player_type_2.as_str() {
            "me" => {
                if self.player_name_2 != auth_user.username {
                    errors.push((2, format!("Me must be you.")));
                }
                CreateMatchFormSelects {
                    i: 2,
                    options: CreateMatchOptions::me(auth_user),
                    selected: Some(auth_user.username.clone()),
                }
            }
            "user" => {
                if self.player_type_2 == auth_user.username {
                    errors.push((2, format!("Select 'me' for yourself.")));
                }
                CreateMatchFormSelects {
                    i: 2,
                    options: CreateMatchOptions::users(auth_user, conn),
                    selected: Some(self.player_name_2.to_string()),
                }
            }
            "agent" => CreateMatchFormSelects {
                i: 2,
                options: CreateMatchOptions::agents(auth_user, conn),
                selected: Some(self.player_name_2.to_string()),
            },
            _ => CreateMatchFormSelects::default(auth_user, 2),
        };

        match (self.player_type_1.as_str(), self.player_type_2.as_str()) {
            ("user", "user") => {
                // Can't create a game between two users that isn't you.
                errors.push((1, format!("wat: {}", self.player_name_1)));
            }
            ("user", "agent") => {
                // Can't create a game between a user that isn't you and an agent.
                errors.push((1, format!("wat: {}", self.player_name_1)));
            }
            ("agent", "user") => {
                // Can't create a game between a user that isn't you and an agent.
                errors.push((2, format!("wat: {}", self.player_name_2)));
            }
            _ => {}
        }

        eprintln!("errors: {:?}", errors);

        if errors.len() > 0 {
            return Err(CreateMatchForm {
                blue: blue_select,
                red: red_select,
                errors,
            });
        }

        Ok(())
    }
}
