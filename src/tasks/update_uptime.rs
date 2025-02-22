use std::time::Duration;

use mongodb::Collection;

use crate::commands::uptime_command::{update_uptime, ApiError, Uptime};

pub async fn uptime_updater(api_key: &str, collection: Collection<Uptime>) -> Result<(), ApiError> {
	loop {
		let players: Vec<String> = collection
			.distinct("uuid", None, None)
			.await?
			.into_iter()
			.filter_map(|bson_value| bson_value.as_str().map(String::from))
			.collect();

		println!("Updating Uptime for {} players", players.len());

		let mut no_guild: u16 = 0;
		for player in players {
			if processed_uuids.contains(&player) {
				continue;
			}

			update_uptime(player, api_key, collection);
		}
		if no_guild > 0 {
			println!(
				"\x1b[34m[INFO] {} players are no longer in a guild\x1b[0m",
				no_guild
			);
		}

		tokio::time::sleep(Duration::from_secs(3 * 60 * 60)).await; // 3 hours
	}
}
