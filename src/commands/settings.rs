//use crate::{Context, Error};
//use poise::serenity_prelude as serenity;
//use poise::CreateReply;
//use serde::{Deserialize};
//use serenity::all::{CreateEmbed, User};
//use serenity::json::Value;
//use std::collections::HashMap;
//use rusqlite::{params, Connection, Result};
//use rusqlite::NO_PARAMS;
//
//#[poise::command(slash_command, ephemeral = true)]
//pub async fn tracking(
//    ctx: Context<'_>,
//    #[description = "enable or disable uptime tracking"] setting: String,
//) -> Result<(), Error> {
//    let conn = Connection::open("users.db")?;
//
//
//}