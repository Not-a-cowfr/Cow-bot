use std::borrow::Cow;
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
			.filter_map(|bson_value| bson_value.as_str().map(ToOwned::to_owned))
			.collect();

		println!("Updating Uptime for {} players", players.len());
		let mut processed_uuids = Vec::with_capacity(players.len());

		let mut no_guild = 0u16;
		for player in players {
			if processed_uuids.contains(&player) {
				continue;
			}

			if let Err(ApiError::NoGuild()) = update_uptime(&player, api_key, client).await {
				no_guild += 1;
			}

			processed_uuids.push(player);
		}

		if no_guild > 0 {
			println!(
				"\x1b[34m[INFO] {} players are no longer in a guild\x1b[0m",
				no_guild
			);
		}

		tokio::time::sleep(Duration::from_secs(10 * 60)).await;
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

#[derive(Deserialize)]
struct Guild {
	members: Vec<Member>,
	#[serde(rename = "_id")]
	id:      String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Member {
	uuid:       String,
	expHistory: Option<Value>,
}

type UptimeHistory = HashMap<String, i64>;
type GuildUptimeData = HashMap<String, UptimeHistory>;

async fn get_guild_uptime_data(
	api_key: &str,
	uuid: &str,
) -> Result<(String, GuildUptimeData), Box<dyn std::error::Error + Send + Sync>> {
	let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");

	let response = reqwest::get(&url).await?;
	let response_text = response.text().await?;
	let guild_response: GuildResponse = serde_json::from_str(&response_text)?;

	let guild = guild_response.guild.ok_or_else(|| ApiError::NoGuild())?;
	let mut guild_uptime_data = HashMap::with_capacity(guild.members.len());

	for member in guild.members {
		if let Some(exp_history) = member.expHistory {
			let uptime_history: UptimeHistory = exp_history
				.as_object()
				.map(|history| {
					history
						.iter()
						.filter_map(|(date, xp)| {
							xp.as_i64().map(|xp_value| (date.to_owned(), xp_value))
						})
						.collect()
				})
				.unwrap_or_default();

			if !uptime_history.is_empty() {
				guild_uptime_data.insert(member.uuid, uptime_history);
			}
		}
	}

	Ok((guild.id, guild_uptime_data))
}

pub async fn update_uptime(
	uuid: &str,
	api_key: &str,
	client: &Client,
) -> Result<(), ApiError> {
	let (guild_id, member_uptime_history) = get_guild_uptime_data(api_key, uuid).await?;

	let collection: Collection<Uptime> = client.database("Players").collection("Uptime");
	let index_model = IndexModel::builder()
		.keys(doc! { "uuid": 1, "date": 1 })
		.options(IndexOptions::builder().unique(true).build())
		.build();
	collection.create_index(index_model).await?;

	let models: Vec<_> = member_uptime_history
		.into_iter()
		.flat_map(|(uuid, uptime_history)| {
			let guild_id = Cow::Borrowed(&guild_id);
			uptime_history.into_iter().map({
				let value = collection.clone();
				move |(unformatted_date, new_gexp)| {
					let date = format!("{} 00:00:00", unformatted_date);
					let naive_date = NaiveDateTime::parse_from_str(&date, "%Y-%m-%d %H:%M:%S")
						.expect("Failed to parse date");
					let bson_date = BsonDateTime::from_chrono(Utc.from_utc_datetime(&naive_date));

					let filter = doc! {
						"uuid": &uuid,
						"date": &bson_date,
					};

					let update = doc! {
						"_id": ObjectId::new(),
						"uuid": &uuid,
						"gexp": new_gexp,
						"date": bson_date,
						"guild_id": guild_id.as_ref(),
					};

					ReplaceOneModel::builder()
						.namespace(value.namespace())
						.filter(filter)
						.replacement(update)
						.upsert(true)
						.build()
				}
			})
		})
		.collect();

	if !models.is_empty() {
		client.bulk_write(models).await?;
	}

	Ok(())
}
