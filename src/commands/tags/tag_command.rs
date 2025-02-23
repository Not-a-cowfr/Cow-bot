use poise::CreateReply;
use serenity::all::CreateEmbed;
use crate::commands::tags::tag_utils::get_data_and_id;
use crate::{Context, Error};

use crate::commands::utils::{create_error_embed, get_color};

#[poise::command(prefix_command, subcommands("create", "edit", "delete", "list"))]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "Tag name"]
    name: String,
) -> Result<(), Error> {
    let is_reply = match &ctx {
        Context::Prefix(prefix_ctx) => prefix_ctx.msg.referenced_message.is_some(),
        _ => false,
    };

    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some(content)) = data.tag_db.get_tag(&name, id).await {
        let mut builder = CreateReply::default().content(content);
        if is_reply {
            builder = builder.reply(true);
        }
        ctx.send(builder).await?;
    } else {
        ctx.send(CreateReply::default().embed(create_error_embed("Tag not found")))
            .await?;
    }

    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn create(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
    #[description = "Tag content"]
    #[rest]
    content: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    match data.tag_db.create_tag(&name, &content, id).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content(format!("Created tag `{}`", name)))
                .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn delete(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    match data.tag_db.delete_tag(&name, id).await {
        Ok(true) => {
            ctx.send(CreateReply::default().content(format!("Deleted tag `{}`", name)))
                .await?
        }
        Ok(false) => {
            ctx.send(CreateReply::default().embed(create_error_embed("Tag not found")))
                .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}

#[poise::command(prefix_command, slash_command)]
async fn edit(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
    #[description = "New content"]
    #[rest]
    content: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    match data.tag_db.edit_tag(&name, &content, id).await {
        Ok(true) => {
            ctx.send(CreateReply::default().content(format!("Updated tag `{}`", name)))
                .await?
        }
        Ok(false) => {
            ctx.send(
                CreateReply::default().embed(create_error_embed("Tag not found")),
            )
            .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}
