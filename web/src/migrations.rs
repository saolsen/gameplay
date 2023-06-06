use lazy_static::lazy_static;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, Result, M};

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![
        M::up(
            r#"
            CREATE TABLE user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                clerk_id TEXT NOT NULL,
                username TEXT NOT NULL,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                email TEXT NOT NULL
            ) STRICT;
            CREATE UNIQUE INDEX idx_user_clerk_id ON user (clerk_id);
        "#
        ),
        M::up("CREATE UNIQUE INDEX idx_user_username ON user (username);"),
        M::up(
            r#"
            CREATE TABLE agent (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL REFERENCES user(id),
                game TEXT CHECK(game IN ('connect4')) NOT NULL DEFAULT 'connect4',
                agentname TEXT NOT NULL
            ) STRICT;
            CREATE UNIQUE INDEX idx_agent_agentname ON agent (agentname);
        "#
        ),
        M::up(
            r#"
            CREATE TABLE match (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game TEXT CHECK(game IN ('connect4')) NOT NULL DEFAULT 'connect4',
                created_by INTEGER NOT NULL REFERENCES user(id),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            ) STRICT;
        "#
        ),
        M::up(
            r#"
            CREATE TABLE match_player (
                match_id INTEGER NOT NULL REFERENCES match(id),
                number INTEGER NOT NULL,
                user_id INTEGER REFERENCES user(id),
                agent_id INTEGER REFERENCES agent(id),
                PRIMARY KEY (match_id, number),
                CHECK(
                    (user_id  IS NULL AND agent_id IS NOT NULL) OR
                    (agent_id IS NULL AND user_id  IS NOT NULL)
                )
            ) STRICT;
        "#
        ),
        M::up(
            r#"
            CREATE TABLE match_turn (
                match_id INTEGER NOT NULL REFERENCES match(id),
                number INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                player INTEGER,
                action TEXT,
                status TEXT CHECK(status IN ('over', 'in_progress')) NOT NULL,
                winner INTEGER,
                next_player INTEGER,
                state TEXT NOT NULL,
                PRIMARY KEY (match_id, number),
                CHECK(
                    (status = 'over' AND next_player IS NULL) OR
                    (status = 'in_progress' AND next_player IS NOT NULL AND winner IS NULL)
                )
            ) STRICT;
        "#
        ),
        M::up(
            r#"
            DROP INDEX idx_agent_agentname;
            CREATE UNIQUE INDEX idx_agent_agentname ON agent (user_id, game, agentname);
        "#
        ),
        M::up(
            r#"
            CREATE TABLE agent_deployment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id INTEGER NOT NULL REFERENCES agent(id),
                url TEXT NOT NULL
            ) STRICT;
            CREATE UNIQUE INDEX idx_agent_deployment_agent_id ON agent_deployment (agent_id);
        "#
        ),
    ]);
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    MIGRATIONS.to_latest(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        MIGRATIONS.validate().unwrap();
    }
}
