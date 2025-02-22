use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use futures::stream::StreamExt;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{Document, doc};
use bson::DateTime as BsonDateTime;
use mongodb::options::ReplaceOneModel;
use mongodb::{Client, Collection, Cursor};
use poise::CreateReply;
use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use serenity::json::Value;


use crate::commands::utils::{get_account_from_anything, get_color};
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn uptime(
	ctx: Context<'_>,
	#[description = "Username, UUID, or discord ID"] user: Option<String>,
	#[description = "Time window, eg 7 for 7 days"] window: Option<i64>,
) -> Result<(), Error> {
	let mut start = Instant::now();
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
	println!("get username/uuid: {} ms", start.elapsed().as_millis());
	start = Instant::now();
	let time_window: i64 = window.unwrap_or(7);

	let api_key = &ctx.data().api_key;
	println!("get collection: {} ms", start.elapsed().as_millis());
	start = Instant::now();

	let uptime_data = match get_uptime(api_key, uuid, time_window, ctx.data().mongo_client.clone()).await {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Uptime {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	id:       Option<ObjectId>,
	uuid:     String,
	gexp:     i64,
	date:     BsonDateTime,
	guild_id: String,
}

#[allow(dead_code)]
async fn get_uptime(
	api_key: &str,
	uuid: String,
	time_window: i64,
	client: Client,
) -> Result<Vec<(String, i64)>, mongodb::error::Error> {
	let mut start = Instant::now();
	update_uptime(uuid.clone(), api_key, client.clone()).await.unwrap();
	println!("update uptime: {} ms", start.elapsed().as_millis());
	start = Instant::now();

	let date: DateTime<Utc> = Utc::now();
	let start_date: BsonDateTime = BsonDateTime::from_chrono(date - Duration::days(time_window));
	let filter: Document = doc! {
		"uuid": uuid,
		"date": { "$gte": start_date }
	};

	
	let mut cursor: Cursor<Uptime> = client.database("Players").collection("Uptime").find(filter).await?;
	let mut results: Vec<(String, i64)> = Vec::new();

	while let Some(result) = cursor.next().await {
		let playtime: Uptime = result?;
		let date_str: String = playtime.date.to_string();
		results.push((date_str, playtime.gexp as i64));
	}
	println!("get uptime results: {} ms", start.elapsed().as_millis());

	Ok(results)
}


#[derive(Debug)]
pub enum ApiError {
	Database(mongodb::error::Error),
	Api(String),
	NoGuild(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		match self {
			| ApiError::Database(e) => write!(f, "Database error: {}", e),
			| ApiError::Api(msg) => write!(f, "API error: {}", msg),
			| ApiError::NoGuild(uuid) => write!(f, "Player {} is not in a guild", uuid),
		}
	}
}

impl From<mongodb::error::Error> for ApiError {
	fn from(err: mongodb::error::Error) -> ApiError { ApiError::Database(err) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ApiError {
	fn from(err: Box<dyn std::error::Error + Send + Sync>) -> ApiError { ApiError::Api(err.to_string()) }
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

async fn get_guild_uptime_data(
	api_key: &str,
	uuid: String,
) -> Result<(String, HashMap<String, HashMap<String, i64>>), Box<dyn std::error::Error + Send + Sync>>
{
	let mut start = Instant::now();
	let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");

	let response = reqwest::get(&url).await?;
	println!("get guild data: {} ms", start.elapsed().as_millis());
	start = Instant::now();
	let response_text = response.text().await?;
	let guild_response: GuildResponse = serde_json::from_str(&response_text)?;

	let guild = guild_response
		.guild
		.ok_or_else(|| ApiError::NoGuild(uuid.clone()))?;
	println!("parse data: {} ms", start.elapsed().as_millis());
	start = Instant::now();

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
	println!("return data: {} ms", start.elapsed().as_millis());

	Ok((guild._id, guild_uptime_data))
}

pub async fn update_uptime(
    uuid: String,
    api_key: &str,
    client: Client,
) -> Result<(), mongodb::error::Error> {
    let (guild_id, member_uptime_history) =
        get_guild_uptime_data(api_key, uuid.clone()).await.unwrap();

    let mut models = Vec::new();
	let collection: Collection<Uptime> = client.database("Players").collection("Uptime");

    for (uuid, uptime_history) in member_uptime_history {
        for (unformatted_date, new_gexp) in uptime_history {
            let formatted_date = format!("{} 00:00:00", unformatted_date);
            let naive_date = NaiveDateTime::parse_from_str(&formatted_date, "%Y-%m-%d %H:%M:%S")
                .expect("Failed to parse date");
            let date = BsonDateTime::from_chrono(Utc.from_utc_datetime(&naive_date));

            let filter = doc! {
                "uuid": uuid.clone(),
                "date": &date,
            };

            let update = doc! {
				"_id": ObjectId::new(),
                "uuid": uuid.clone(),
				"gexp": new_gexp,
				"date": date,
				"guild_id": guild_id.clone(),
            };

			let model = ReplaceOneModel::builder()
					.namespace(collection.namespace())
					.filter(filter)
					.replacement(update)
					.upsert(true)
					.build();

            models.push(model);
        }
    }

    if !models.is_empty() {
        client.bulk_write(models).await?;
    }

    Ok(())
}

#[allow(dead_code)]
fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}
