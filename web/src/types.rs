use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: i64,
    pub clerk_id: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct AgentRecord {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    Connect4,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    pub game: Game,
    pub username: String,
    pub agentname: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Player {
    User(User),
    Agent(Agent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect4Action {
    pub column: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect4State {
    pub board: Vec<Option<usize>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Over{ winner: Option<usize> },
    InProgress{ next_player: usize },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Turn<A> {
    pub number: usize,
    pub player: Option<usize>,
    pub action: Option<A>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Match<A, S> {
    pub id: i64,
    pub game: Game,
    pub players: Vec<Player>,
    pub turns: Vec<Turn<A>>,
    pub turn: usize,
    pub state: S,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let m = Match {
            id: 123,
            game: Game::Connect4,
            players: vec![
                Player::User(User {
                    username: "user1".to_string(),
                }),
                Player::Agent(Agent {
                    game: Game::Connect4,
                    username: "user2".to_string(),
                    agentname: "agent1".to_string(),
                }),
            ],
            turns: vec![
                Turn {
                    number: 0,
                    player: None,
                    action: None,
                    status: Status::InProgress{ next_player: 0 },
                },
                Turn {
                    number: 1,
                    player: Some(0),
                    action: Some(Connect4Action { column: 0 }),
                    status: Status::InProgress{ next_player: 1 },
                },
            ],
            turn: 0,
            state: Connect4State{
                board: vec![None; 42],
            }
        };
        eprintln!("{}", serde_json::to_string(&m).unwrap());
    }
}