use std::pin::Pin;
use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use futures::stream::StreamExt;
use mongodb::bson::{Document, doc};
use bson::DateTime as BsonDateTime;
use mongodb::{Client, Cursor};
use poise::CreateReply;
use serenity::builder::CreateEmbed;

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::tasks::update_uptime::{ApiError, Uptime, update_uptime};
use crate::{Context, Error, API_KEY, MONGO_CLIENT};

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

	let client = MONGO_CLIENT.get().expect("MongoDB client is uninitalized").clone();
	let uptime_data = match get_uptime(uuid, time_window, client).await {
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

	let mut description = String::new();
	for (date, gexp) in uptime_data {
		let uptime_str = if gexp >= 0 {
			gexp_to_uptime_as_string(gexp)
		} else {
			"Unknown".to_string()
		};
		description.push_str(&format!("{}: {}\n", BsonDateTime_to_string(date), uptime_str));
	}

	let color = get_color(&ctx.author().name);
	let embed = CreateEmbed::default()
		.title(format!("Uptime for {}", username))
		.description(description)
		.color(color);

	ctx.send(CreateReply::default().embed(embed)).await?;
	println!("time taken: {} ms", start.elapsed().as_millis());
	Ok(())
}

fn get_uptime(
    uuid: String,
    time_window: i64,
    client: Client,
) -> Pin<Box<dyn Future<Output = Result<Vec<(String, i64)>, ApiError>> + Send>> {
    Box::pin(async move {
        let date: DateTime<Utc> = Utc::now();
        let start_date: BsonDateTime =
            BsonDateTime::from_chrono(date - Duration::days(time_window));
        let filter: Document = doc! {
            "uuid": uuid.clone(),
            "date": { "$gte": start_date }
        };

        let mut cursor: Cursor<Uptime> = client
            .database("Players")
            .collection("Uptime")
            .find(filter.clone())
            .await?;
        let mut results: Vec<(String, i64)> = Vec::new();

        while let Some(result) = cursor.next().await {
            let playtime: Uptime = result?;
            let date_str: String = playtime.date.to_string();
            results.push((date_str, playtime.gexp as i64));
        }

        if results.is_empty() {
			update_uptime(
				uuid.clone(),
				API_KEY.get().expect("API_KEY is uninitialized"),
				client.clone(),
			).await?;
			
			let mut cursor = client
				.database("Players")
				.collection("Uptime")
				.find(filter)
				.await?;
			results.clear();
			while let Some(result) = cursor.next().await {
				let playtime: Uptime = result?;
				let date_str = playtime.date.to_string();
				results.push((date_str, playtime.gexp as i64));
			}
		}

        Ok(results)
    })
}

fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}

#[allow(non_snake_case)]
fn BsonDateTime_to_string(date: String) -> String {
	format!("**{}**", date.replace(" 0:00:00.0 +00:00:00", ""))
}
