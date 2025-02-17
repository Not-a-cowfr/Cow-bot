use poise::CreateReply;
use serenity::all::{CreateEmbed, User};

use crate::commands::utils::{get_account_from_anything, get_color};
use crate::{Context, Error};

#[poise::command(
	slash_command,
	context_menu_command = "Get Linked Account",
	ephemeral = true
)]
pub async fn get_linked_account(
	ctx: Context<'_>,
	#[description = "Discord profile to get linked account of"] user: User,
) -> Result<(), Error> {
	let (username, uuid) = match get_account_from_anything(&user.id.to_string()).await {
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

	let color = get_color(&ctx.author().name);

	let embed = CreateEmbed::default()
        .title(format!("Player information for **{username}**"))
        .description(format!(
            "Username: **{username}**\nUUID: `{uuid}`\n\n<https://elitebot.dev/@{username}>\n\n<https://sky.shiiyu.moe/stats/{username}>"
        ))
        .color(color);

	ctx.send(CreateReply::default().embed(embed)).await?;
	Ok(())
}
