use poise::CreateReply;
use rusqlite::{Connection, params};
use serenity::builder::CreateEmbed;

#[allow(deprecated)]
use crate::commands::utils::{get_account_from_anything_elite, get_color};
use crate::{Context, Error};

/// Link your minecraft account to the bot for easier usage
#[poise::command(slash_command, prefix_command)]
pub async fn link(
	ctx: Context<'_>,
	#[description = "username/uuid"] name: Option<String>,
) -> Result<(), Error> {
	let user = &ctx.author().name;
	let user_id = &ctx.author().id.to_string();
	let identifier = name.unwrap_or_else(|| user_id.clone());

	#[allow(deprecated)]
	let (username, uuid) = get_account_from_anything_elite(&identifier).await?;

	{
		let conn = Connection::open("src/data/users.db")?;

		let user_exists = {
			let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE id = ?1")?;
			let mut rows = stmt.query(params![user_id])?;
			rows.next()?.unwrap().get::<_, i64>(0)? > 0
		};

		if user_exists {
			conn.execute(
				"UPDATE users SET mc_username = ?1, mc_uuid = ?2 WHERE id = ?3",
				params![username, uuid, user_id],
			)?;
		} else {
			conn.execute(
				"INSERT INTO users (username, id, mc_username, mc_uuid) VALUES (?1, ?2, ?3, ?4)",
				params![user, user_id, username, uuid],
			)?;
		}
	}

	let color = get_color(&user);
	let embed = CreateEmbed::default()
		.title("Account Linked!")
		.description(format!(
			"Your account, {}, has been linked successfully!",
			username
		))
		.color(color);
	ctx.send(CreateReply::default().embed(embed)).await?;

	Ok(())
}
