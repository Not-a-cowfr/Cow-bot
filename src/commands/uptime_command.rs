use std::collections::HashMap;
use std::pin::Pin;

use bson::DateTime as BsonDateTime;
use chrono::{Duration, Utc};
use futures::stream::StreamExt;
use mongodb::bson::doc;
use mongodb::{Client, Cursor};
use poise::CreateReply;
use serenity::builder::CreateEmbed;
use tokio::time::Instant;

use crate::commands::utils::{create_error_embed, get_account_from_anything, get_color};
use crate::tasks::update_uptime::{ApiError, Uptime, update_uptime};
use crate::{API_KEY, Context, Error, MONGO_CLIENT};

#[poise::command(slash_command, prefix_command, invoke_on_edit, reuse_response)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] user: Option<String>,
	#[description = "Time window, eg 7 for 7 days"] window: Option<i64>,
) -> Result<(), Error> {
	let start = Instant::now();
	ctx.defer().await?;

	let user_id = user.unwrap_or_else(|| ctx.author().id.to_string());
	let (username, uuid) = match get_account_from_anything(&user_id).await {
		| Ok(result) => result,
		| Err(_e) => {
			let embed = create_error_embed("No linked account found");
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};
	let time_window = window.unwrap_or(7);

	let client = MONGO_CLIENT
		.get()
		.expect("MongoDB client is uninitalized")
		.clone();
	let mut uptime_data = match get_uptime(&uuid, time_window, client).await {
		| Ok(uptime_data) => uptime_data,
		| Err(e) => {
			println!("{}", e);
			let embed = create_error_embed(&e.to_string());
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};

	if uptime_data.len() < time_window as usize {
		uptime_data = fill_missing_dates(uptime_data, time_window);
	}

	let mut description = String::with_capacity(2_000);
	for (date, gexp) in uptime_data {
		let uptime = if gexp == -1 {
			"Unknown".to_string()
		} else {
			gexp_to_uptime_as_string(gexp)
		};
		description.push_str(&format!("{}: {}\n", BsonDateTime_to_string(&date), uptime));
	}

	let color = get_color(&ctx.author().name);
	let embed = CreateEmbed::default()
		.title(format!("Uptime for {username}"))
		.description(description)
		.color(color);

	ctx.send(CreateReply::default().embed(embed)).await?;
	println!(
		"Uptime command for {} took {} ms",
		username,
		start.elapsed().as_millis()
	);
	Ok(())
}

fn get_uptime(
	uuid: &str,
	time_window: i64,
	client: Client,
) -> Pin<Box<dyn Future<Output = Result<Vec<(BsonDateTime, i64)>, ApiError>> + Send + '_>> {
	Box::pin(async move {
		let date = Utc::now();
		let start_date = BsonDateTime::from_chrono(date - Duration::days(time_window));
		let filter = doc! {
			"uuid": uuid,
			"date": { "$gte": start_date }
		};

		let mut cursor: Cursor<Uptime> = client
			.database("Players")
			.collection("Uptime")
			.find(filter.clone())
			.await?;
		let mut results = Vec::new();

		while let Some(result) = cursor.next().await {
			let playtime = result?;
			results.push((playtime.date, playtime.gexp as i64));
		}

		if results.is_empty() {
			update_uptime(
				uuid,
				API_KEY.get().expect("API_KEY is uninitialized"),
				&client,
			)
			.await?;

			let mut cursor: Cursor<Uptime> = client
				.database("Players")
				.collection("Uptime")
				.find(filter)
				.await?;
			results.clear();
			while let Some(result) = cursor.next().await {
				let playtime = result?;
				results.push((playtime.date, playtime.gexp as i64));
			}
		}

		Ok(results)
	})
}

fn fill_missing_dates(
	mut results: Vec<(BsonDateTime, i64)>,
	time_window: i64,
) -> Vec<(BsonDateTime, i64)> {
	let now = Utc::now();
	let start_date = now - Duration::days(time_window);

	let date_map: HashMap<String, i64> = results
		.iter()
		.map(|(date, gexp)| (date.to_chrono().format("%Y-%m-%d").to_string(), *gexp))
		.collect();

	for i in results.len()..time_window as usize {
		let current_date = start_date + Duration::days(i as i64);
		let bson_date = BsonDateTime::from_chrono(current_date);
		let normalized_date = current_date.format("%Y-%m-%d").to_string();

		let gexp = date_map.get(&normalized_date).copied().unwrap_or(-1);
		results.push((bson_date, gexp));
	}

	results
}

fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}

#[allow(non_snake_case)]
fn BsonDateTime_to_string(date: &BsonDateTime) -> String {
	format!(
		"**{}**",
		date.to_string().get(..10).unwrap_or("Unknown Date")
	)
}
