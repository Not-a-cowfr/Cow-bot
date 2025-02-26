use rusqlite::{Connection, Result, params};
use serde::Deserialize;
use serenity::all::CreateEmbed;

use crate::ERROR_COLOR;
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

	color_result
		.ok()
		.flatten()
		.and_then(|color_str| u32::from_str_radix(color_str.trim_start_matches("0x"), 16).ok())
		.unwrap_or(0x383838) // default color if there's an error or no color found
}

#[derive(Deserialize)]
struct MojangResponse {
	id:   String,
	name: String,
}

pub async fn get_account_from_anything(identifier: &str) -> Result<(String, String), Error> {
	let clean_identifier = identifier
		.replace(&['@', '<', '>'][..], "")
		.trim()
		.to_string();

	let result = if identifier.len() == 32 || identifier.len() <= 16 {
		get_mojang_info(identifier.to_string()).await?
	} else if clean_identifier.parse::<u64>().is_ok() {
		get_linked_elite_account(clean_identifier).await?
	} else {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Invalid player name or UUID",
		)));
	};

	Ok(result)
}

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

	if !response.status().is_success() {
		let error_text = response.text().await?;
		if error_text.contains("CONSTRAINT_VIOLATION") {
			return Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::InvalidInput,
				"Invalid UUID string",
			)));
		} else {
			return Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				format!("Player \"{}\" does not exist", player),
			)));
		}
	}

	let mojang_info: MojangResponse = response.json().await?;
	Ok((mojang_info.name, mojang_info.id))
}

pub async fn get_linked_elite_account(discord_id: String) -> Result<(String, String), Error> {
	let url = format!("https://api.elitebot.dev/account/{}", discord_id);
	let response = reqwest::get(&url).await?;
	let status = response.status();
	let body = response.text().await?;

	if !status.is_success() {
		if body.trim() == "Minecraft account not found." {
			return Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::NotFound,
				"No linked account found!",
			)));
		} else {
			return Err(Box::new(std::io::Error::new(
				std::io::ErrorKind::Other,
				format!("Error: {}", body),
			)));
		}
	}

	let account: MojangResponse = serde_json::from_str(&body)?;
	Ok((account.name, account.id))
}

pub fn create_error_embed(description: &str) -> CreateEmbed {
	CreateEmbed::default()
		.title("Error")
		.description(description)
		.color(*ERROR_COLOR.get().expect("ERROR_COLOR is uninitialized"))
}
