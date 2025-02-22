use std::collections::HashMap;
use std::fmt;
use std::time::Duration;

use bson::oid::ObjectId;
use bson::{DateTime as BsonDateTime, Document, doc};
use chrono::{NaiveDateTime, TimeZone, Utc};
use mongodb::options::{IndexOptions, ReplaceOneModel};
use mongodb::{Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::MONGO_CLIENT;

#[derive(Debug, Serialize, Deserialize)]
pub struct Uptime {
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id:       Option<ObjectId>,
	pub uuid:     String,
	pub gexp:     i64,
	pub date:     BsonDateTime,
	pub guild_id: String,
}

pub async fn uptime_updater(
	api_key: &str,
	collection: Collection<Uptime>,
) -> Result<(), ApiError> {
	loop {
		let client = MONGO_CLIENT.get().unwrap();

		let players: Vec<String> = collection
			.distinct("uuid", Document::new())
			.await?
			.into_iter()
			.filter_map(|bson_value| bson_value.as_str().map(String::from))
			.collect();

		println!("Updating Uptime for {} players", players.len());
		let mut processed_uuids: Vec<String> = Vec::new();

		let mut no_guild: u16 = 0;
		for player in players {
			if processed_uuids.contains(&player.clone()) {
				continue;
			}

			match update_uptime(player.clone(), api_key, client.clone()).await {
				| Err(ApiError::NoGuild()) => no_guild += 1,
				| _ => {},
			};

			processed_uuids.push(player);
		}
		if no_guild > 0 {
			println!(
				"\x1b[34m[INFO] {} players are no longer in a guild\x1b[0m",
				no_guild
			);
		}

		tokio::time::sleep(Duration::from_secs(10 * 60)).await; // 10 minutes
	}
}

#[derive(Debug)]
pub enum ApiError {
	Database(mongodb::error::Error),
	Api(String),
	NoGuild(),
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
			| ApiError::NoGuild() => write!(f, "Player is not in a guild"),
		}
	}
}

impl From<mongodb::error::Error> for ApiError {
	fn from(err: mongodb::error::Error) -> ApiError { ApiError::Database(err) }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ApiError {
	fn from(err: Box<dyn std::error::Error + Send + Sync>) -> ApiError {
		ApiError::Api(err.to_string())
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

async fn get_guild_uptime_data(
	api_key: &str,
	uuid: String,
) -> Result<(String, HashMap<String, HashMap<String, i64>>), Box<dyn std::error::Error + Send + Sync>>
{
	let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");

	let response = reqwest::get(&url).await?;
	let response_text = response.text().await?;
	let guild_response: GuildResponse = serde_json::from_str(&response_text)?;

	let guild = guild_response
		.guild
		.ok_or_else(|| ApiError::NoGuild())?;

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

	Ok((guild._id, guild_uptime_data))
}

pub async fn update_uptime(
	uuid: String,
	api_key: &str,
	client: Client,
) -> Result<(), ApiError> {
	let (guild_id, member_uptime_history) = match get_guild_uptime_data(api_key, uuid.clone()).await
	{
		| Ok(result) => result,
		| Err(e) => return Err(e.into()),
	};

	let mut models = Vec::new();
	let collection: Collection<Uptime> = client.database("Players").collection("Uptime");
	let index_model = IndexModel::builder()
		.keys(doc! { "uuid": 1, "date": 1 })
		.options(IndexOptions::builder().unique(true).build())
		.build();
	collection.create_index(index_model).await?;

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
