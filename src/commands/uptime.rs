use std::collections::{HashMap, HashSet};
use std::time::Instant;

use chrono::{Duration, Utc};
use poise::CreateReply;
use rusqlite::{Connection, params};
use serenity::builder::CreateEmbed;

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::update_uptime::get_guild_uptime_data;
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] mut user: Option<String>,
	#[description = "Time window, eg 7 for 7 days"] mut window: Option<i64>,
) -> Result<(), Error> {
	let start = Instant::now();
	ctx.defer().await?;

	let user: String = user.unwrap_or_else(|| ctx.author().id.to_string());
	let time_window: i64 = window.unwrap_or(7);

	let api_key = &ctx.data().api_key;

	let author = ctx.author();
	let color = get_color(&author.name);

	let uptime_data = match get_uptime(&user, time_window).await {
		| Ok(data) => data,
		| Err(_) => match update_uptime(api_key, user.clone()).await {
			| Ok(_) => match get_uptime(&user, time_window).await {
				| Ok(data) => data,
				| Err(_) => get_uptime_with_unknown(&user, time_window).await?,
			},
			| Err(_) => get_uptime_with_unknown(&user, time_window).await?,
		},
	};

	let mut description = String::new();
	for (date, gexp) in uptime_data {
		let uptime_str = if gexp >= 0 {
			gexp_to_uptime_as_string(gexp)
		} else {
			"Unknown".to_string()
		};
		description.push_str(&format!("{}: {}\n", date, uptime_str));
	}

	let (username, _uuid) = get_account_from_anything(&user).await?;
	let embed = CreateEmbed::default()
		.title(format!("Uptime for {}", username))
		.description(description)
		.color(color);

	ctx.send(CreateReply::default().embed(embed)).await?;

	println!("time taken for command: {:?}", start.elapsed());
	Ok(())
}

async fn get_uptime_with_unknown(
	user: &str,
	time_window: i64,
) -> Result<Vec<(String, i64)>, Error> {
	let (_, uuid) = get_account_from_anything(&user).await?;

	let conn = Connection::open("src/data/uptime.db")?;

	let current_date = Utc::now().date_naive();

	let mut results = Vec::new();
	for days_ago in 0..time_window {
		let date = current_date - Duration::days(days_ago);
		results.push((date.format("%Y-%m-%d").to_string(), -1));
	}

	let mut stmt = conn.prepare(
		"SELECT date, gexp FROM uptime
            WHERE uuid = ?
            AND date >= date('now', ? || ' days')
            ORDER BY date DESC",
	)?;

	let rows = stmt.query_map([uuid, format!("-{}", time_window - 1)], |row| {
		Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
	})?;

	let mut date_to_gexp = HashMap::new();
	for row in rows {
		if let Ok((date, gexp)) = row {
			date_to_gexp.insert(date, gexp);
		}
	}

	for result in results.iter_mut() {
		if let Some(&gexp) = date_to_gexp.get(&result.0) {
			result.1 = gexp;
		}
	}

	Ok(results)
}

async fn get_uptime(
	user: &str,
	time_window: i64,
) -> Result<Vec<(String, i64)>, Error> {
	let (_, uuid) = get_account_from_anything(&user).await?;

	let conn = Connection::open("src/data/uptime.db")?;

	let current_date = Utc::now().date_naive();

	let mut expected_dates = HashSet::new();
	for days_ago in 0..time_window {
		let date = current_date - Duration::days(days_ago);
		expected_dates.insert(date.format("%Y-%m-%d").to_string());
	}

	let mut stmt = conn.prepare(
		"SELECT date, gexp FROM uptime
            WHERE uuid = ?
            AND date >= date('now', ? || ' days')
            ORDER BY date DESC",
	)?;

	let rows = stmt.query_map([uuid, format!("-{}", time_window - 1)], |row| {
		Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
	})?;

	let mut found_dates = HashSet::new();
	let mut results = Vec::new();

	for row in rows {
		if let Ok((date, gexp)) = row {
			found_dates.insert(date.clone());
			results.push((date, gexp));
		}
	}

	let missing_dates: Vec<_> = expected_dates.difference(&found_dates).collect();

	if !missing_dates.is_empty() {
		return Err("Missing dates".into());
	}

	Ok(results)
}

async fn update_uptime(
	api_key: &str,
	player: String,
) -> Result<(), Error> {
	let conn = Connection::open("src/data/uptime.db")?;

	let (guild_id, member_uptime_history) = get_guild_uptime_data(api_key, player).await?;

	for (player_uuid, uptime_history) in member_uptime_history {
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

	Ok(())
}

pub fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}
