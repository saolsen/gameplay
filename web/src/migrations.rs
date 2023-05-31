use lazy_static::lazy_static;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, Result, M};

lazy_static! {
    static ref MIGRATIONS: Migrations<'static> = Migrations::new(vec![M::up(
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
    ),]);
}

pub fn migrate(conn: &mut Connection) -> Result<()> {
    MIGRATIONS.to_latest(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }
}
