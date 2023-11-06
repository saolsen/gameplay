use rusqlite::{params, OptionalExtension};

use crate::types;

pub fn get_by_id(
    conn: &types::Conn,
    match_id: i64,
) -> Option<types::Match<types::Connect4Action, types::Connect4State>> {
    let match_ = conn
        .query_row(
            r#"
                SELECT
                    match.id,
                    match_turn.number as turn,
                    match_turn.status,
                    match_turn.winner,
                    match_turn.next_player,
                    match_turn.state
                FROM match
                JOIN match_turn ON match.id = match_turn.match_id
                WHERE match.id = ?
                AND match.game = 'connect4'
                AND match_turn.number = (
                    SELECT max(number)
                    FROM match_turn
                    WHERE match_id = match.id
                )
            "#,
            [match_id],
            |row| {
                let id: i64 = row.get(0)?;
                let turn: usize = row.get(1)?;
                let status_str: String = row.get(2)?;
                let winner: Option<usize> = row.get(3)?;
                let next_player: Option<usize> = row.get(4)?;
                let state: String = row.get(5)?;

                let status = match status_str.as_str() {
                    "in_progress" => {
                        let next_player = next_player.unwrap();
                        types::Status::InProgress { next_player }
                    }
                    "over" => types::Status::Over { winner },
                    _ => unreachable!(),
                };

                let state: types::Connect4State = serde_json::from_str(&state).unwrap();

                Ok((id, turn, status, state))
            },
        )
        .optional()
        .unwrap();

    if let Some((id, turn, status, state)) = match_ {
        let mut player_stmt = conn
            .prepare(
                r#"
                    SELECT
                        match_player.number,
                        match_player.user_id,
                        match_player.agent_id,
                        user.username as user_username,
                        agent_user.username as agent_username,
                        agent.agentname as agent_agentname
                    FROM match_player
                    LEFT JOIN user ON user.id = match_player.user_id
                    LEFT JOIN agent ON agent.id = match_player.agent_id
                    LEFT JOIN user AS agent_user ON agent_user.id = agent.user_id
                    WHERE match_id = ?
                    ORDER BY number ASC
                "#,
            )
            .unwrap();
        let match_players = player_stmt
            .query_map([match_id], |row| {
                let number: i64 = row.get(0)?;
                let user_id: Option<i64> = row.get(1)?;
                let agent_id: Option<i64> = row.get(2)?;
                let user_username: Option<String> = row.get(3)?;
                let agent_username: Option<String> = row.get(4)?;
                let agent_agentname: Option<String> = row.get(5)?;

                let player = if user_id.is_some() {
                    types::Player::User(types::User {
                        username: user_username.unwrap(),
                    })
                } else if agent_id.is_some() {
                    types::Player::Agent(types::Agent {
                        game: types::Game::Connect4,
                        username: agent_username.unwrap(),
                        agentname: agent_agentname.unwrap(),
                    })
                } else {
                    panic!("match_player has neither user_id nor agent_id")
                };
                Ok((number, player))
            })
            .unwrap();
        let mut players = vec![];
        for (i, player) in match_players.enumerate() {
            let (n, p) = player.unwrap();
            assert_eq!(i, n as usize);
            players.push(p);
        }

        let mut turn_stmt = conn
            .prepare(
                r#"
                    SELECT
                        number,
                        player,
                        action
                    FROM match_turn
                    WHERE match_id = ?
                    ORDER BY number ASC
                "#,
            )
            .unwrap();
        let match_turns = turn_stmt
            .query_map([match_id], |row| {
                let number: usize = row.get(0)?;
                let player: Option<usize> = row.get(1)?;
                let action_json: Option<String> = row.get(2)?;

                let action = match action_json {
                    Some(action_json) => {
                        let action: types::Connect4Action =
                            serde_json::from_str(&action_json).unwrap();
                        Some(action)
                    }
                    None => None,
                };

                Ok(types::Turn {
                    number,
                    player,
                    action,
                })
            })
            .unwrap();
        let mut turns = vec![];
        for (i, turn) in match_turns.enumerate() {
            let t = turn.unwrap();
            assert_eq!(i, t.number);
            turns.push(t);
        }

        let match_ = types::Match {
            id,
            game: types::Game::Connect4,
            players,
            turns,
            turn,
            status,
            state,
        };
        return Some(match_);
    }
    None
}

pub enum PlayerId {
    User(i64),
    Agent(i64),
}

pub fn create(
    conn: &mut types::Conn,
    created_by_user_id: i64,
    blue_player: PlayerId,
    red_player: PlayerId,
) -> i64 {
    // New Match
    let turns: Vec<types::Turn<types::Connect4Action>> = vec![types::Turn {
        number: 0,
        player: None,
        action: None,
    }];
    let state = types::Connect4State {
        board: vec![None; 42],
    };

    let (blue_player_user_id, blue_player_agent_id) = match blue_player {
        PlayerId::User(user_id) => (Some(user_id), None),
        PlayerId::Agent(agent_id) => (None, Some(agent_id)),
    };
    let (red_player_user_id, red_player_agent_id) = match red_player {
        PlayerId::User(user_id) => (Some(user_id), None),
        PlayerId::Agent(agent_id) => (None, Some(agent_id)),
    };

    let match_id = {
        let tx = conn.transaction().unwrap();
        tx.execute(
            r#"
                INSERT INTO match (game, created_by)
                VALUES (?, ?)
            "#,
            params!["connect4", created_by_user_id],
        )
        .unwrap();
        let match_id = tx.last_insert_rowid();
        tx.execute(
            r#"
                INSERT INTO match_player (match_id, number, user_id, agent_id)
                VALUES (?, ?, ?, ?)
            "#,
            params![
                // blue player
                match_id,
                0,
                blue_player_user_id,
                blue_player_agent_id,
            ],
        )
        .unwrap();
        tx.execute(
            r#"
                INSERT INTO match_player (match_id, number, user_id, agent_id)
                VALUES (?, ?, ?, ?)
            "#,
            params![
                // red player
                match_id,
                1,
                red_player_user_id,
                red_player_agent_id
            ],
        )
        .unwrap();
        tx.execute(
            r#"
                INSERT INTO match_turn (match_id, number, player, action, status, winner, next_player, state)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                match_id,
                turns[0].number,
                turns[0].player,
                None::<String>,
                "in_progress",
                None::<usize>,
                Some(0),
                serde_json::to_string(&state).unwrap(),
                ],
        ).unwrap();

        tx.commit().unwrap();
        match_id
    };

    match_id
}

pub fn user_turn(_conn: &types::Conn) {
    unimplemented!()
}

pub fn agent_turn(_conn: &types::Conn) {
    unimplemented!()
}
