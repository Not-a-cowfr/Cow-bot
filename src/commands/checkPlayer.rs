use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::CreateReply;
use serde::Deserialize;
use serenity::all::{CreateEmbed, User};
use serenity::json::Value;
use std::collections::HashMap;

// command(s)
#[poise::command(slash_command, context_menu_command = "Get Linked Account")]
pub async fn get_linked_account(
    ctx: Context<'_>,
    #[description = "Discord profile to get linked account of"] user: User,
) -> Result<(), Error> {
    ctx.defer().await?;
    let (username, uuid) = get_account_from_anything(user.id.to_string()).await?;

    let color = 0xa10d0d;
    let embed = CreateEmbed::default()
        .title(format!("Player information for **{username}**"))
        .description(format!(
            "Username: **{username}**\nUUID: `{uuid}`\n\n<https://elitebot.dev/@{username}>"
        ))
        .colour(color);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command, context_menu_command = "Check Player")]
pub async fn check_player(
    ctx: Context<'_>,
    #[description = "Player to check"] user: User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let player = user.id.to_string();
    let api_key = &ctx.data().api_key;
    let (uptime_history, avg_uptime) = get_uptime(api_key, &player).await?;

    let mut uptime_hist = String::new();
    if uptime_history.is_empty() {
        uptime_hist.push_str("An error occurred");
    } else {
        for (date, uptime) in &uptime_history {
            uptime_hist.push_str(&format!("{}: {}\n", date, uptime));
        }
        uptime_hist.push_str(&format!("**Average Uptime**: {}\n", avg_uptime));
    }

    let embed = CreateEmbed::default()
        .title(format!("Farming stats for **{}**", user.name))
        .field("Uptime History", uptime_hist, false)
        .colour(0xa10d0d);

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

// Utils
#[derive(Deserialize)]
struct MojangResponse {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct GuildResponse {
    success: bool,
    guild: Option<Guild>,
}

#[derive(Deserialize)]
struct Guild {
    members: Vec<Member>,
}

#[derive(Deserialize)]
struct Member {
    uuid: String,
    expHistory: Option<Value>,
}

async fn get_mojang_info(player: String) -> Result<(String, String), Error> {
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
    if (identifier.len() == 32) | (identifier.len() <= 16) {
        // mojang uuid or username
        let result = get_mojang_info(identifier.into()).await?;
        username = result.0;
        uuid = result.1;
    } else if identifier
        .replace(&['@', '<', '>'][..], "")
        .trim()
        .parse::<u64>()
        .is_ok()
    {
        // discord id
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

async fn get_uptime(
    api_key: &str,
    identifier: &str,
) -> Result<(HashMap<String, String>, String), Box<dyn std::error::Error + Send + Sync>> {
    let (username, uuid) = get_account_from_anything(identifier.to_string()).await?;
    let url = format!("https://api.hypixel.net/v2/guild?key={api_key}&player={uuid}");
    let response = reqwest::get(&url).await?;
    let guild_response: GuildResponse = response.json().await?;

    if guild_response.guild.is_none() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Player is not in a guild",
        )));
    }

    for member in guild_response.guild.unwrap().members {
        if member.uuid == uuid {
            let mut uptime_history = HashMap::new();
            let mut total_xp = 0;

            if let Some(exp_history) = member.expHistory {
                for (date, xp) in exp_history.as_object().unwrap() {
                    let formatted_date =
                        format!("{}/{}/{}", &date[8..10], &date[5..7], &date[2..4]);
                    let xp_value = xp.as_i64().unwrap();
                    total_xp += xp_value;
                    let formatted_xp = format!("{}h {}m", xp_value / 9000, (xp_value % 9000) / 150);
                    uptime_history.insert(formatted_date, formatted_xp);
                }
            }

            let avg_uptime = format!("{}h {}m", total_xp / 7 / 9000, (total_xp / 7 % 9000) / 150);
            return Ok((uptime_history, avg_uptime));
        }
    }

    Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Player not found",
    )))
}
