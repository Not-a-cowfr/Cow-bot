use poise::{CreateReply, serenity_prelude as serenity};
use rusqlite::{Connection, params};
use serenity::all::CreateEmbed;

use super::utils::create_error_embed;
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command, invoke_on_edit, reuse_response)]
pub async fn color(
	ctx: Context<'_>,
	#[description = "Hex code"] color: String,
) -> Result<(), Error> {
	let user = ctx.author();
	let user_id = user.id.to_string();
	let username = &user.name;

	let color = color.replace("#", "");

	if !color.chars().all(|c| c.is_digit(16)) || color.len() != 6 {
		let embed =
			create_error_embed("Invalid hex code. Please provide a valid 6-character hex code.");
		ctx.send(CreateReply::default().embed(embed)).await?;
		return Ok(());
	}

	let color_value = u32::from_str_radix(&color, 16).unwrap();
	let color_but_with_thingy = format!("0x{}", color);

	let user_exists;
	{
		let conn = Connection::open("src/data/users.db")?;
		let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE userid = ?1")?;
		let mut rows = stmt.query(params![user_id])?;
		user_exists = rows.next()?.unwrap().get::<_, i64>(0)? > 0;

		if user_exists {
			conn.execute(
				"UPDATE users SET color = ?1 WHERE userid = ?2",
				params![color_but_with_thingy, user_id],
			)?;
		} else {
			conn.execute(
				"INSERT INTO users (username, userid, color) VALUES (?1, ?2, ?3)",
				params![*username, user_id, color_but_with_thingy],
			)?;
		}
	}

	let embed = CreateEmbed::default()
		.title("Color Updated")
		.description("Your color has been updated successfully!")
		.color(color_value);
	ctx.send(CreateReply::default().embed(embed)).await?;
	Ok(())
}
