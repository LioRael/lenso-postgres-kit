//! Ensure a Host can compose PostgreSQL Plugins with its own SQLite storage.
#[test]
fn sqlite_and_postgres_can_share_a_process() {
    let sqlite = rusqlite::Connection::open_in_memory().unwrap();
    let value: i64 = sqlite.query_row("SELECT 42", [], |row| row.get(0)).unwrap();
    assert_eq!(value, 42);
    let _postgres = lenso_postgres_kit::sqlx::postgres::PgConnectOptions::new();
}
