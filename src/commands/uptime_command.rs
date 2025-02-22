use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use futures::stream::StreamExt;
use mongodb::bson::{Document, doc};
use bson::DateTime as BsonDateTime;
use mongodb::{Collection, Cursor};
use poise::CreateReply;
use serenity::builder::CreateEmbed;

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::tasks::update_uptime::Uptime;
use crate::{Context, Error, MONGO_CLIENT};

#[poise::command(slash_command)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] user: Option<String>,
	#[description = "Time window, eg 7 for 7 days"] window: Option<i64>,
) -> Result<(), Error> {
	ctx.defer().await?;

	let user_input = user.unwrap_or_else(|| ctx.author().id.to_string());
	let mut start = Instant::now();
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
	println!("get username/uuid: {} ms", start.elapsed().as_millis());
	start = Instant::now();
	let time_window: i64 = window.unwrap_or(7);

	println!("get collection: {} ms", start.elapsed().as_millis());
	start = Instant::now();

	let collection = MONGO_CLIENT.get().expect("MongoDB client is uninitalized").clone().database("Players").collection("Uptime");
	let uptime_data = match get_uptime(uuid, time_window, collection).await {
		| Ok(uptime_data) => uptime_data,
		| Err(e) => {
			println!("{}", e);
			let embed = CreateEmbed::default()
				.title("Unexpected Error occured")
				.description(e.to_string())
				.color(ctx.data().error_color);
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};
	println!(
		"Fetched uptime successfully: {} ms",
		start.elapsed().as_millis()
	);
	start = Instant::now();

	let mut description = String::new();
	for (date, gexp) in uptime_data {
		let uptime_str = if gexp >= 0 {
			gexp_to_uptime_as_string(gexp)
		} else {
			"Unknown".to_string()
		};
		description.push_str(&format!("{}: {}\n", date, uptime_str));
	}
	println!("generate description: {} ms", start.elapsed().as_millis());
	start = Instant::now();

	let color = get_color(&ctx.author().name);
	let embed = CreateEmbed::default()
		.title(format!("Uptime for {}", username))
		.description(description)
		.color(color);
	println!("make embed: {} ms", start.elapsed().as_millis());
	start = Instant::now();

	ctx.send(CreateReply::default().embed(embed)).await?;
	log::info!("time taken: {:?}", start.elapsed());
	println!("reply: {} ms", start.elapsed().as_millis());
	Ok(())
}

async fn get_uptime(
	uuid: String,
	time_window: i64,
	collection: Collection<Uptime>,
) -> Result<Vec<(String, i64)>, mongodb::error::Error> {
	let date: DateTime<Utc> = Utc::now();
	let start_date: BsonDateTime = BsonDateTime::from_chrono(date - Duration::days(time_window));
	let filter: Document = doc! {
		"uuid": uuid,
		"date": { "$gte": start_date }
	};

	
	let mut cursor: Cursor<Uptime> = collection.find(filter).await?;
	let mut results: Vec<(String, i64)> = Vec::new();

	while let Some(result) = cursor.next().await {
		let playtime: Uptime = result?;
		let date_str: String = playtime.date.to_string();
		results.push((date_str, playtime.gexp as i64));
	}

	Ok(results)
}

#[allow(dead_code)]
fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}
