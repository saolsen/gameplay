use askama::Template;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "create_agent_form.html")]
pub struct CreateAgentForm {
    pub game: String,
    pub agentname: String,
    pub url: String,
    pub agentname_error: Option<String>,
    pub url_error: Option<String>,
}

impl CreateAgentForm {
    pub fn default() -> Self {
        Self {
            game: "connect4".to_owned(),
            agentname: "".to_owned(),
            url: "".to_owned(),
            agentname_error: None,
            url_error: None,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CreateAgentFormData {
    pub game: String,
    pub agentname: String,
    pub url: String,
}
