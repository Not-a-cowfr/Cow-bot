use std::pin::Pin;
use std::time::Instant;

use chrono::{DateTime, Duration, Utc};
use futures::stream::StreamExt;
use mongodb::bson::{Document, doc};
use bson::DateTime as BsonDateTime;
use mongodb::{Client, Cursor};
use poise::CreateReply;
use serenity::builder::CreateEmbed;

use crate::commands::utils::{create_error_embed, get_account_from_anything, get_color};
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
			let embed = create_error_embed("No linked account found");
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};
	let time_window: i64 = window.unwrap_or(7);

	let client = MONGO_CLIENT.get().expect("MongoDB client is uninitalized").clone();
	let mut uptime_data = match get_uptime(uuid, time_window, client).await {
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
		let uptime: String;
		if gexp == -1 {
			uptime = "Unkown".to_string()
		} else {
			uptime = gexp_to_uptime_as_string(gexp)
		}
		description.push_str(&format!("{}: {}\n", BsonDateTime_to_string(&date), uptime));
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
            "uuid": &uuid,
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
				&uuid,
				API_KEY.get().expect("API_KEY is uninitialized"),
				&client,
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

fn fill_missing_dates(
    mut results: Vec<(String, i64)>,
    time_window: i64,
) -> Vec<(String, i64)> {
    let now = Utc::now();
    let start_date = now - Duration::days(time_window);

    let date_map: std::collections::HashMap<String, i64> = results
        .iter()
        .map(|(date, gexp)| (BsonDateTime_to_string(date), *gexp))
        .collect();

    for i in results.len()..time_window as usize {
        let current_date = start_date + Duration::days(i as i64);
        let date_str = BsonDateTime::from_chrono(current_date).to_string();
        let normalized_date = current_date.format("%Y-%m-%d").to_string();

        let gexp = date_map.get(&normalized_date).copied().unwrap_or(-1);
        results.push((date_str, gexp));
    }

    results
}

#[allow(dead_code)]
fn gexp_to_uptime_as_string(gexp: i64) -> String {
	format!("{}h {}m", gexp / 9000, (gexp % 9000) / 150)
}

#[allow(non_snake_case, dead_code)]
fn BsonDateTime_to_string(date: &String) -> String {
	format!("**{}**", date.get(..10).unwrap_or("Unknown Date"))
}
