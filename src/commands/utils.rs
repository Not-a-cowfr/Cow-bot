use rusqlite::{Connection, Result, params};
use serde::Deserialize;
use serenity::all::CreateEmbed;

use crate::ERROR_COLOR;
use crate::tasks::update_uptime::ApiError;
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
	// TODO: combine these 2 functions into 1
	let color_result = get_color_backend(username);

	color_result
		.ok()
		.flatten()
		.and_then(|color_str| u32::from_str_radix(color_str.trim_start_matches("0x"), 16).ok())
		.unwrap_or(0x2b2d31) // color of discord embed with default discord dark theme
}

#[derive(Deserialize)]
struct MojangResponse {
	id:   String,
	name: String,
}

#[deprecated = "use get_account_from_anything"]
pub async fn get_account_from_anything_elite(identifier: &str) -> Result<(String, String), Error> {
	let clean_identifier = identifier
		.replace(&['@', '<', '>'][..], "")
		.trim()
		.to_string();

	let result = if identifier.len() == 32 || identifier.len() <= 16 {
		get_mojang_info(identifier.to_string()).await?
	} else if clean_identifier.parse::<u64>().is_ok() {
		match get_account_from_anything(&clean_identifier).await {
			| Ok(result) => return Ok(result),
			| Err(_) => get_linked_elite_account(clean_identifier).await?,
		}
	} else {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Invalid player name or UUID",
		)));
	};

	Ok(result)
}

pub async fn get_account_from_anything(identifier: &str) -> Result<(String, String), Error> {
	let clean_identifier = identifier
		.replace(&['@', '<', '>'][..], "")
		.trim()
		.to_string();

	let result = if identifier.len() == 32 || identifier.len() <= 16 {
		get_mojang_info(identifier.to_string()).await?
	} else if clean_identifier.parse::<u64>().is_ok() {
		get_linked_account(clean_identifier).await?
	} else {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Invalid player name or UUID",
		)));
	};

	Ok(result)
}

pub async fn get_linked_account(id: String) -> Result<(String, String), Error> {
	let conn = Connection::open("src/data/users.db")?;

	let mut stmt = conn.prepare("SELECT mc_username, mc_uuid FROM users WHERE id = ?1")?;
	let mut rows = stmt.query([id])?;

	if let Some(row) = rows.next()? {
		Ok((row.get(0)?, row.get(1)?))
	} else {
		Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"No linked account found",
		)))
	}
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

#[derive(Deserialize, Debug)]
struct PlayerResponse {
	success: bool,
	player:  Option<Player>,
	#[serde(default)]
	cause:   Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
struct Player {
	socialMedia: Option<SocialMedia>,
}

#[derive(Deserialize, Debug)]
struct SocialMedia {
	links: Links,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case, dead_code)]
pub struct Links {
	TWITTER:   Option<String>,
	YOUTUBE:   Option<String>,
	INSTAGRAM: Option<String>,
	TWITCH:    Option<String>,
	DISCORD:   Option<String>,
	FORUMS:    Option<String>,
}

pub async fn get_hypixel_linked_socials(uuid: String) -> Result<Links, Error> {
	let api_key = std::env::var("API_KEY")?;
	let url = format!(
		"https://api.hypixel.net/v2/player?key={}&uuid={}",
		api_key, uuid
	);

	let response = reqwest::get(&url).await?;
	let player_data: PlayerResponse = response.json().await?;

	if !player_data.success {
		if let Some(e) = &player_data.cause {
			return Err(Box::new(ApiError::Api(e.clone())));
		}
	}

	if !player_data.success {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::Other,
			player_data
				.cause
				.unwrap_or_else(|| "Unknown error".to_string()),
		)));
	}

	match player_data.player {
		| Some(player) => match player.socialMedia {
			| Some(social_media) => Ok(social_media.links),
			| None => Ok(Links {
				TWITTER:   None,
				YOUTUBE:   None,
				INSTAGRAM: None,
				TWITCH:    None,
				DISCORD:   None,
				FORUMS:    None,
			}),
		},
		| None => Ok(Links {
			TWITTER:   None,
			YOUTUBE:   None,
			INSTAGRAM: None,
			TWITCH:    None,
			DISCORD:   None,
			FORUMS:    None,
		}),
	}
}

pub async fn is_hypixel_linked_account(
	uuid: String,
	discord_user: String,
) -> Result<bool, Error> {
	let linked_socials = get_hypixel_linked_socials(uuid).await?;

	if linked_socials.DISCORD.is_none() {
		return Err(Box::new(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"Please link your discord on Hypixel".to_string(),
		)));
	}

	Ok(linked_socials
		.DISCORD
		.map_or(false, |discord| discord == discord_user))
}

pub fn create_error_embed(description: &str) -> CreateEmbed {
	CreateEmbed::default()
		.title("Error")
		.description(description)
		.color(*ERROR_COLOR.get().expect("ERROR_COLOR is uninitialized"))
}
