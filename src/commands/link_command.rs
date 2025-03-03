use poise::CreateReply;
use rusqlite::{Connection, params};
use serenity::builder::CreateEmbed;

#[allow(deprecated)]
use crate::commands::utils::{
	create_error_embed,
	get_account_from_anything_elite,
	get_color,
	is_hypixel_linked_account,
};
use crate::{Context, Error};

/// Link your minecraft account to the bot for easier usage
#[poise::command(slash_command, prefix_command)]
pub async fn link(
	ctx: Context<'_>,
	#[description = "username/uuid"] name: String,
) -> Result<(), Error> {
	let user = &ctx.author().name;
	let user_id = &ctx.author().id.to_string();

	#[allow(deprecated)]
	match get_account_from_anything_elite(&name).await {
		| Ok((username, uuid)) => {
			match is_hypixel_linked_account(uuid.clone(), user.clone()).await {
				| Err(e) => {
					ctx.send(
                        CreateReply::default()
                            .embed(
                                create_error_embed(&e.to_string())
                                    .image("https://media.discordapp.net/attachments/922202066653417512/1066476136953036800/tutorial.gif")
                            )
                    )
                    .await?;
					return Ok(());
				},
				| Ok(false) => {
					ctx.send(CreateReply::default().embed(create_error_embed(
						"You cannot link to accounts that are not yours",
					)))
					.await?;
					return Ok(());
				},
				| _ => {},
			}

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
		},
		| Err(e) => {
			ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string())))
				.await?;
			Ok(())
		},
	}
}
