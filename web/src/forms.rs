use serde::{Deserialize};
use crate::types;

// 2 fields, red and blue
// type is me, user or agent
// value is a name, that is either the user's username, or the agent's name
// then there's the data for the selects in the form, which depends on the type that is selected.
// so it's like, type, value and options, and those depend on the type, value is sort of secondary.

// template for the selects, takes a type, an optional value and the options.
// template for the whole form has two of those selects.
// form that gets posted is just the types and values.

pub enum CreateMatchOptions {
    Me(String),
    User(Vec<String>),
    Agent(Vec<String>),
}

pub struct SelectsTemplate {
    pub options: CreateMatchOptions,
    pub selected: Option<String>,
}

pub struct CreateMatchFormTemplate {
    pub blue: SelectsTemplate,
    pub red: SelectsTemplate,
}

#[derive(Deserialize, Debug)]
pub struct CreateMatchFormData {
    pub player_type_1: String,
    pub player_name_1: String,
    pub player_type_2: String,
    pub player_name_2: String,
}

pub fn create_match_selects(
    auth_user: &types::UserRecord,
    player_type: &str,
    player_name: Option<&str>,
    n: i32,
) {
    let player = {
        match n {
            1 => "blue",
            2 => "red",
            _ => panic!("invalid n"),
        }
    };

    let selects = match player_type {
        "me" => {
            format!(
                r#"<input name="player_name_{}" type="hidden" value="{}">"#,
                n,
                auth_user.username
            )
        }
        "user" => {
            let options = vec![
                format!(r#"<option value="{}">{}</option>"#, "steveo", "steveo"),
                format!(r#"<option value="{}">{}</option>"#, "gabe", "gabe"),
            ];

            format!(
                r#"
                <label for="{}_player" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">username</label>
                <select name="player_name_{}" id="{}_player" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {}
                </select>
                "#,
                player,
                n,
                player,
                options.join("\n")
            )
        }
        "agent" => {
            let options = vec![
                format!(
                    r#"<option value="{}/{}">{}/{}</option>"#,
                    "steveo", "random", "steveo", "random"
                ),
                format!(
                    r#"<option value="{}/{}">{}/{}</option>"#,
                    "gabe", "smart", "gabe", "smart"
                ),
            ];

            format!(
                r#"
                <label for="{}_player" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">agentname</label>
                <select name="player_name_{}" id="{}_player" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500">
                    {}
                </select>
                "#,
                player,
                n,
                player,
                options.join("\n")
            )
        }
        _ => {
            panic!("invalid player type")
        }
    };
}