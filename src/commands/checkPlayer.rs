use crate::{Context, Error};
use serde::Deserialize;
use poise::CreateReply;
use serenity::all::{CreateEmbed, User};
use poise::serenity_prelude as serenity;


// command(s)

#[poise::command(slash_command, context_menu_command = "Get Linked Account", )]
pub async fn get_linked_account(
    ctx: Context<'_>,
    #[description = "Discord profile to get linked account of"] user: User,
) -> Result<(), Error> {
    ctx.defer().await?;
    let (username, uuid) = get_account_from_anything(user.id.to_string()).await?;

    let color = 0xa10d0d;
    let embed = CreateEmbed::default()
        .title(format!("Player information for **{username}**"))
        .description(format!("Username: **{username}**\nUUID: `{uuid}`\n\n<https://elitebot.dev/@{username}>"))
        .colour(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}



// Utils
#[derive(Deserialize)]
struct MojangResponse {
    id: String,
    name: String,
}

async fn http_get_mojang_info(player: String) -> Result<(String, String), Error> {
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

async fn get_linked_elite_account(discordid: String) -> Result<(String, String), Error> {
    let url = format!("https://api.elitebot.dev/account/{discordid}");
    let response = reqwest::get(&url).await?;
    let mojang_info: MojangResponse = response.json().await?;
    Ok((mojang_info.name, mojang_info.id))
}

async fn get_account_from_anything(identifier: String) -> Result<(String, String), Error> {
    let (uuid, username);
    if (identifier.len() == 32) | (identifier.len() <= 16) { // mojang uuid or username
        let result = http_get_mojang_info(identifier.into()).await?;
        username = result.0;
        uuid = result.1;
    } else if identifier.replace(&['@', '<', '>'][..], "").trim().parse::<u64>().is_ok() { // discord id
        let result = get_linked_elite_account(identifier.into()).await?;
        username = result.0;
        uuid = result.1;
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid player name or UUID",
        )));
    }
    Ok((username, uuid))
}
