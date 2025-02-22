use rusqlite::{Connection, Result};

pub fn create_users_table() -> Result<()> {
	let conn = Connection::open("src/data/users.db")?;
	conn.execute(
		"CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY,
			username TEXT NOT NULL,
			userid TEXT NOT NULL,
			color TEXT NOT NULL
			)",
		[],
	)?;
	Ok(())
}
