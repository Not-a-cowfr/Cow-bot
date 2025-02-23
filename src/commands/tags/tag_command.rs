use poise::CreateReply;
use serenity::all::CreateEmbed;
use crate::commands::tags::tag_utils::get_data_and_id;
use crate::{Context, Error};

use crate::commands::utils::{create_error_embed, get_color};

#[poise::command(prefix_command, slash_command, subcommands("create", "edit", "delete", "list", "preview", "raw"), invoke_on_edit, reuse_response)]
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

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
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

#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
pub async fn create(
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

#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
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

#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
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

#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
async fn list(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    let color = get_color(&ctx.author().name);

    match data.tag_db.get_all_tags(id).await {
        Ok(tags) => {
            let formatted_tags = if tags.is_empty() {
                "No tags found. try creating a tag with `/tag create`".to_string()
            } else {
                tags.join(", ")
            };
    
            ctx.send(CreateReply::default().embed(
                CreateEmbed::default()
                    .title("All Commands")
                    .description(formatted_tags)
                    .color(color),
            ))
            .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}

#[poise::command(slash_command, invoke_on_edit, reuse_response)]
async fn preview(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
        ctx.send(CreateReply::default().content(content).ephemeral(true)).await?;
    } else {
        ctx.send(
            CreateReply::default().embed(create_error_embed("Tag not found")).ephemeral(true)
        ).await?;
    }
    Ok(())
}

#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
async fn raw(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
        ctx.send(CreateReply::default().content(content.replace("`", "\\`"))).await?;
    } else {
        ctx.send(CreateReply::default().embed(create_error_embed("Tag not found")))
            .await?;
    }

    Ok(())
}
