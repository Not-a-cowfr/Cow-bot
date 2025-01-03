use std::collections::HashMap;

use poise::{CreateReply, serenity_prelude as serenity};
use serde::Deserialize;
use serenity::all::CreateEmbed;
use serenity::json::Value;

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] mut user: Option<String>,
) -> Result<(), Error> {
	ctx.defer().await?;

	if user.is_none() {
		user = Some(ctx.author().id.to_string());
	}

	let api_key = &ctx.data().api_key;

	let mut uptime_hist = String::new();
	let (uptime_history, avg_uptime) = match get_uptime(api_key, user.as_deref()).await {
		| Ok(result) => result,
		| Err(e) => {
			uptime_hist.push_str(&format!("{}\n", e));
			(HashMap::new(), String::new())
		},
	};

	if !uptime_history.is_empty() {
		for (date, uptime) in &uptime_history {
			uptime_hist.push_str(&format!("`{}`: {}\n", date, uptime));
		}
		uptime_hist.push_str(&format!("\n**Average Uptime**: {}\n", avg_uptime));
	}

	let (username, _uuid) = match get_account_from_anything(user.as_deref().unwrap()).await {
		| Ok(result) => result,
		| Err(_e) => {
			let embed = CreateEmbed::default()
				.title("Error")
				.description("Cannot find an account. Did you input a user mention?")
				.colour(0xa10d0d);
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};

	let author = ctx.author();
	let color = get_color(&author.name);

	let embed = CreateEmbed::default()
		.title(format!("Uptime for **{}**", username.clone()))
		.field("Uptime History\n", uptime_hist, true)
		.colour(color);

	ctx.send(CreateReply::default().embed(embed)).await?;
	Ok(())
}

#[derive(Deserialize)]
struct GuildResponse {
	guild: Option<Guild>,
}

#[derive(Deserialize)]
struct Guild {
	members: Vec<Member>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Member {
	uuid:       String,
	expHistory: Option<Value>,
}

async fn get_uptime(
	api_key: &str,
	identifier: Option<&str>,
) -> Result<(HashMap<String, String>, String), Box<dyn std::error::Error + Send + Sync>> {
	let (_username, uuid) = get_account_from_anything(identifier.unwrap()).await?;
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

	for member in guild_response.guild.unwrap().members {
		if member.uuid == uuid {
			let mut uptime_history = HashMap::new();
			let mut total_xp = 0;

			if let Some(exp_history) = member.expHistory {
				for (date, xp) in exp_history.as_object().unwrap() {
					let formatted_date =
						format!("{}/{}/{}", &date[8..10], &date[5..7], &date[2..4]);
					let xp_value = xp.as_i64().unwrap();
					total_xp += xp_value;
					let formatted_xp = format!("{}h {}m", xp_value / 9000, (xp_value % 9000) / 150);
					uptime_history.insert(formatted_date, formatted_xp);
				}
			}

			let avg_uptime = format!("{}h {}m", total_xp / 7 / 9000, (total_xp / 7 % 9000) / 150);
			return Ok((uptime_history, avg_uptime));
		}
	}

	Err(Box::new(std::io::Error::new(
		std::io::ErrorKind::NotFound,
		"Player not found",
	)))
}
