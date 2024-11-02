use crate::{Context, Error};
use serde::Deserialize;
use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateEmbedFooter};

#[poise::command(prefix_command, slash_command)]
pub async fn get_mojang_info(
    ctx: Context<'_>,
    #[description = "uuid or username of the player"] user: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let (username, uuid) = http_get_mojang_info(user).await?;

    let color = 0xa10d0d;
    // NOTE embeds are like literally the same as discord.py lol
    let embed = CreateEmbed::default()
        .title(format!("Player information for **{username}**"))
        .description(format!("Username: **{username}**\nUUID: `{uuid}`"))
        .footer(CreateEmbedFooter::new("Bot by Not a cow"))
        .colour(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(()) // Add this line to return the expected Result type
}

#[derive(Deserialize)]
struct MojangResponse {
    id: String,
    name: String,
}

async fn http_get_mojang_info(player: Option<String>) -> Result<(String, String), Error> {
    let player = player.unwrap_or_default();
    let url = if player.len() == 32 {
        format!("https://api.mojang.com/user/profile/{}", player)
    } else if player.len() <= 16 {
        format!("https://api.mojang.com/users/profiles/minecraft/{}", player)
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid player name or UUID",
        )));
    };

    let response = reqwest::get(&url).await?;
    let mojang_info: MojangResponse = response.json().await?;
    Ok((mojang_info.name, mojang_info.id))
}