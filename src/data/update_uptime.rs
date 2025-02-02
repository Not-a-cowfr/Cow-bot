use std::collections::{HashMap, HashSet};
use std::time::Duration;

use rusqlite::{Connection, Result, params};
use serde::Deserialize;
use serenity::json::Value;

use crate::commands::utils::get_account_from_anything;

#[path = "../commands/uptime.rs"]
mod uptime;

#[derive(Debug)]
pub enum Error {
	DatabaseError(()),
	ApiError(()),
}

impl From<rusqlite::Error> for Error {
	fn from(_err: rusqlite::Error) -> Error { Error::DatabaseError(()) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
	fn from(_err: Box<dyn std::error::Error + Send + Sync>) -> Error { Error::ApiError(()) }
}

// todo use guild id instead of this fuckery with uuids and removing ones weve updated already
pub async fn update_uptime(api_key: &str) -> std::result::Result<(), Error> {
	let conn = Connection::open("src/data/uptime.db")?;

	loop {
		let mut processed_uuids = HashSet::new();

		let mut stmt = conn.prepare("SELECT DISTINCT uuid FROM uptime")?;
		let players: Vec<String> = stmt
			.query_map([], |row| row.get(0))?
			.filter_map(Result::ok)
			.collect();

		println!("Updating Uptime for {} players", players.len());

		for player in players {
			if processed_uuids.contains(&player) {
				continue;
			}

			let (guild_id, member_uptime_history) = get_guild_uptime_data(api_key, player).await?;

			for (player_uuid, uptime_history) in member_uptime_history {
				processed_uuids.insert(player_uuid.clone());

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
			}
		}

		tokio::time::sleep(Duration::from_secs(86400)).await; // 24 hours
	}
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

pub async fn get_guild_uptime_data(
	api_key: &str,
	identifier: String,
) -> std::result::Result<
	(String, HashMap<String, HashMap<String, i64>>),
	Box<dyn std::error::Error + Send + Sync>,
> {
	let (_username, uuid) = get_account_from_anything(&identifier).await?;
	let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");
	let response = reqwest::get(&url).await?;
	let response_text = response.text().await?;
	let guild_response: GuildResponse = serde_json::from_str(&response_text)?;

	if guild_response.guild.is_none() {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Player is not in a guild",
		)));
	}

	let mut guild_uptime_data = HashMap::new();

	for member in guild_response.guild.clone().unwrap().members {
		let mut uptime_history = HashMap::new();

		if let Some(ref exp_history) = member.expHistory {
			for (date, xp) in exp_history.as_object().unwrap() {
				let xp_value = xp.as_i64().unwrap();
				uptime_history.insert(date.to_string(), xp_value);
			}
		}
		guild_uptime_data.insert(member.uuid, uptime_history);
	}

	Ok((guild_response.guild.unwrap()._id, guild_uptime_data))
}
