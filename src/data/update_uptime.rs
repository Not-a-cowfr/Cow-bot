use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Duration;

use rusqlite::{Connection, Result as SqliteResult, params};
use serde::Deserialize;
use serenity::json::Value;

use crate::commands::utils::get_account_from_anything;

#[path = "../commands/uptime_command.rs"]
mod uptime;

#[derive(Debug)]
pub enum Error {
	Database(rusqlite::Error),
	Api(String),
	NoGuild(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		match self {
			| Error::Database(e) => write!(f, "Database error: {}", e),
			| Error::Api(msg) => write!(f, "API error: {}", msg),
			| Error::NoGuild(uuid) => write!(f, "Player {} is not in a guild", uuid),
		}
	}
}

impl From<rusqlite::Error> for Error {
	fn from(err: rusqlite::Error) -> Error { Error::Database(err) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
	fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Error { Error::Api(err.to_string()) }
}

#[derive(Deserialize)]
struct GuildResponse {
	guild: Option<Guild>,
}

#[derive(Clone, Deserialize)]
struct Guild {
	members: Vec<Member>,
	_id:     String,
}

#[derive(Clone, Deserialize)]
#[allow(non_snake_case)]
struct Member {
	uuid:       String,
	expHistory: Option<Value>,
}

pub async fn update_uptime(api_key: &str) -> Result<(), Error> {
	let conn = Connection::open("src/data/uptime.db")?;
	loop {
		let mut processed_uuids = HashSet::new();
		let mut stmt = conn.prepare("SELECT DISTINCT uuid FROM uptime")?;
		let players: Vec<String> = stmt
			.query_map([], |row| row.get(0))?
			.filter_map(SqliteResult::ok)
			.collect();

		println!("Updating Uptime for {} players", players.len());

		let mut no_guild: u16 = 0;
		for player in players {
			if processed_uuids.contains(&player) {
				continue;
			}

			match get_guild_uptime_data(api_key, player.clone()).await {
				| Ok((guild_id, member_uptime_history)) => {
					for (player_uuid, uptime_history) in member_uptime_history {
						processed_uuids.insert(player_uuid.clone());
						update_player_records(&conn, &guild_id, &player_uuid, uptime_history)?;
					}
				},
				| Err(_) => {
					no_guild += 1;
					continue;
				},
			}
		}
		if no_guild > 0 {
			println!(
				"\x1b[34m[INFO] {} players are no longer in guild\x1b[0m",
				no_guild
			);
		}

		tokio::time::sleep(Duration::from_secs(3 * 60 * 60)).await; // 3 hours
	}
}

fn update_player_records(
	conn: &Connection,
	guild_id: &str,
	player_uuid: &str,
	uptime_history: HashMap<String, i64>,
) -> Result<(), Error> {
	for (date, gexp) in uptime_history {
		conn.execute(
			"UPDATE uptime
             SET guild_id = ?1, gexp = ?3
             WHERE uuid = ?2 AND date = ?4",
			params![guild_id, player_uuid, gexp, date],
		)?;

		conn.execute(
			"INSERT INTO uptime (guild_id, uuid, gexp, date)
             SELECT ?1, ?2, ?3, ?4
             WHERE NOT EXISTS (
                 SELECT 1 FROM uptime WHERE uuid = ?2 AND date = ?4
             )",
			params![guild_id, player_uuid, gexp, date],
		)?;
	}
	Ok(())
}

pub async fn get_guild_uptime_data(
	api_key: &str,
	identifier: String,
) -> Result<(String, HashMap<String, HashMap<String, i64>>), Box<dyn std::error::Error + Send + Sync>>
{
	let (_username, uuid) = get_account_from_anything(&identifier).await?;
	let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");

	let response = reqwest::get(&url).await?;
	let response_text = response.text().await?;
	let guild_response: GuildResponse = serde_json::from_str(&response_text)?;

	let guild = guild_response
		.guild
		.ok_or_else(|| Error::NoGuild(uuid.clone()))?;

	let mut guild_uptime_data = HashMap::new();

	for member in guild.members {
		let mut uptime_history = HashMap::new();

		if let Some(ref exp_history) = member.expHistory {
			for (date, xp) in exp_history.as_object().unwrap() {
				let xp_value = xp.as_i64().unwrap();
				uptime_history.insert(date.to_string(), xp_value);
			}
		}
		guild_uptime_data.insert(member.uuid, uptime_history);
	}

	Ok((guild._id, guild_uptime_data))
}
