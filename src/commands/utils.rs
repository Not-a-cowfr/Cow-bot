use rusqlite::{params, Connection, Result};
use serde::Deserialize;

use crate::types::Error;

fn get_color_backend(username: &str) -> Result<Option<String>> {
	let conn = Connection::open("src/data/users.db")?;
	let mut stmt = conn.prepare("SELECT color FROM users WHERE username = ?1")?;
	let mut rows = stmt.query(params![username])?;

	if let Some(row) = rows.next()? {
		let color: String = row.get(0)?;
		Ok(Some(color))
	} else {
		Ok(None)
	}
}

pub fn get_color(username: &str) -> u32 {
	let color_result = get_color_backend(username);

	match color_result {
		| Ok(Some(color_str)) => {
			u32::from_str_radix(&color_str.replace("0x", ""), 16).unwrap_or(0x383838)
		},
		| _ => 0x383838, // default color if there's an error or no color found
	}
}

pub async fn get_account_from_anything(identifier: &str) -> Result<(String, String), Error> {
	let (uuid, username);
	if identifier.len() == 32 || identifier.len() <= 16 {
		// mojang uuid or username
		let result = get_mojang_info(identifier.to_string()).await?;
		username = result.0;
		uuid = result.1;
	} else if identifier
		.replace(&['@', '<', '>'][..], "")
		.trim()
		.parse::<u64>()
		.is_ok()
	{
		// discord id
		let result = get_linked_elite_account(identifier.to_string()).await?;
		username = result.0;
		uuid = result.1;
	} else {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Invalid player name or UUID",
		)));
	}
	Ok((username, uuid))
}

#[derive(Deserialize)]
struct MojangResponse {
	id: String,
	name: String,
}

// returns (username, uuid)
pub async fn get_mojang_info(player: String) -> Result<(String, String), Error> {
	let url = if player.len() == 32 {
		format!("https://api.mojang.com/user/profile/{}", player)
	} else if player.len() <= 16 {
		format!("https://api.mojang.com/users/profiles/minecraft/{}", player)
	} else {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Invalid player name or UUID",
		)));
	};

	let response = reqwest::get(&url).await?;
	let mojang_info: MojangResponse = response.json().await?;
	Ok((mojang_info.name, mojang_info.id))
}

pub async fn get_linked_elite_account(discordid: String) -> Result<(String, String), Error> {
	let url = format!("https://api.elitebot.dev/account/{discordid}");
	let response = reqwest::get(&url).await?;
	let mojang_info: MojangResponse = response.json().await?;
	Ok((mojang_info.name, mojang_info.id))
}
