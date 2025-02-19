use std::collections::{HashMap, HashSet};
use std::time::Instant;

use chrono::{Duration, Utc};
use poise::CreateReply;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serenity::builder::CreateEmbed;

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::update_uptime::get_guild_uptime_data;
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] user: Option<String>,
	#[description = "Time window, eg 7 for 7 days"] window: Option<i64>,
) -> Result<(), Error> {
	let start = Instant::now();
	ctx.defer().await?;

	let user_input = user.unwrap_or_else(|| ctx.author().id.to_string());
	let (username, uuid) = match get_account_from_anything(&user_input).await {
		| Ok(result) => result,
		| Err(_e) => {
			let embed = CreateEmbed::default()
				.title("Error")
				.description("No linked account found")
				.color(ctx.data().error_color);
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};
	let time_window: i64 = window.unwrap_or(7);

	let api_key = &ctx.data().api_key;
	let pool = &ctx.data().uptime_db_pool;

	let uptime_data = match get_uptime(&uuid, time_window, pool).await {
		| Ok(data) => data,
		| Err(_) => match update_uptime(api_key, uuid.clone(), pool).await {
			| Ok(_) => match get_uptime(&uuid, time_window, pool).await {
				| Ok(data) => data,
				| Err(_) => get_uptime_with_unknown(&uuid, time_window, pool).await?,
			},
			| Err(_) => get_uptime_with_unknown(&uuid, time_window, pool).await?,
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

	let color = get_color(&ctx.author().name);
	let embed = CreateEmbed::default()
		.title(format!("Uptime for {}", username))
		.description(description)
		.color(color);

	ctx.send(CreateReply::default().embed(embed)).await?;
	log::info!("time taken: {:?}", start.elapsed());
	Ok(())
}

#[allow(dead_code)]
async fn get_uptime_with_unknown(
	uuid: &str,
	time_window: i64,
	pool: &Pool<SqliteConnectionManager>,
) -> Result<Vec<(String, i64)>, Error> {
	let conn = pool.get()?;
	let current_date = Utc::now().date_naive();
	let mut results = Vec::with_capacity(time_window as usize);

	let mut stmt = conn.prepare(
		"SELECT date, gexp FROM uptime
         WHERE uuid = ? AND date >= date('now', '-' || (? - 1) || ' days')
         ORDER BY date DESC",
	)?;

	let mut date_to_gexp = HashMap::new();
	for row in stmt.query_map(params![uuid, time_window], |row| {
		Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
	})? {
		let (date, gexp) = row?;
		date_to_gexp.insert(date, gexp);
	}

	for days_ago in 0..time_window {
		let date = (current_date - Duration::days(days_ago))
			.format("%Y-%m-%d")
			.to_string();
		results.push((date.clone(), *date_to_gexp.get(&date).unwrap_or(&-1)));
	}

	Ok(results)
}

#[allow(dead_code)]
async fn get_uptime(
	uuid: &str,
	time_window: i64,
	pool: &Pool<SqliteConnectionManager>,
) -> Result<Vec<(String, i64)>, Error> {
	let conn = pool.get()?;
	let mut stmt = conn.prepare(
		"SELECT date, gexp FROM uptime
         WHERE uuid = ? AND date >= date('now', '-' || (? - 1) || ' days')
         ORDER BY date DESC",
	)?;

	let rows = stmt.query_map(params![uuid, time_window], |row| {
		Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
	})?;

	let mut results = Vec::new();
	let mut found_dates = HashSet::new();

	for row in rows {
		let (date, gexp) = row?;
		found_dates.insert(date.clone());
		results.push((date, gexp));
	}

	if results.len() < time_window as usize {
		return Err("Missing dates".into());
	}

	Ok(results)
}

#[allow(dead_code)]
async fn update_uptime(
	api_key: &str,
	uuid: String,
	pool: &Pool<SqliteConnectionManager>,
) -> Result<(), Error> {
	let mut conn = pool.get()?;
	let (guild_id, member_uptime_history) = get_guild_uptime_data(api_key, uuid.clone()).await?;

	let tx = conn.transaction()?;

	for (player_uuid, uptime_history) in member_uptime_history {
		for (date, gexp) in uptime_history {
			tx.execute(
				"UPDATE uptime
                    SET guild_id = ?1, gexp = ?3
                    WHERE uuid = ?2 AND date = ?4",
				params![guild_id, player_uuid, gexp, date],
			)?;

			tx.execute(
				"INSERT INTO uptime (guild_id, uuid, gexp, date)
                    SELECT ?1, ?2, ?3, ?4
                    WHERE NOT EXISTS (
                        SELECT 1 FROM uptime WHERE uuid = ?2 AND date = ?4
                    )",
				params![guild_id, player_uuid, gexp, date],
			)?;
		}
	}

	tx.commit()?;

	Ok(())
}

#[allow(dead_code)]
pub fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}
