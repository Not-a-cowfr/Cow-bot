use poise::CreateReply;
use serenity::all::{CreateEmbed, User};

use crate::commands::utils::{create_error_embed, get_account_from_anything, get_color};
use crate::{Context, Error};

#[poise::command(context_menu_command = "Get Linked Account", ephemeral = true)]
pub async fn get_linked_account(
	ctx: Context<'_>,
	#[description = "Discord profile to get linked account of"] user: User,
) -> Result<(), Error> {
	let (username, uuid) = match get_account_from_anything(&user.id.to_string()).await {
		| Ok(result) => result,
		| Err(_e) => {
			let embed = create_error_embed("No linked account found");
			ctx.send(CreateReply::default().embed(embed)).await?;
			return Ok(());
		},
	};

	let color = get_color(&ctx.author().name);

	let embed = CreateEmbed::default()
    .title(format!("Player information for **{username}**"))
    .description(format!(
        "Username: **{username}**\nUUID: `{uuid}`\n\n<https://elitebot.dev/@{username}>\n\n<https://cupcake.shiiyu.moe/stats/{username}>"
    ))
    .color(color)
    .thumbnail(format!("https://mc-heads.net/body/{}/left", uuid));

	ctx.send(CreateReply::default().embed(embed)).await?;
	Ok(())
}
