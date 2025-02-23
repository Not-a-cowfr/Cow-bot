use poise::CreateReply;
use serenity::all::{CreateEmbed, CreateMessage};
use crate::commands::tags::tag_utils::get_data_and_id;
use crate::{Context, Error};

use crate::commands::utils::{create_error_embed, get_color};

#[poise::command(prefix_command, slash_command, subcommands("create", "edit", "delete", "list", "preview", "raw", "alias"), invoke_on_edit, reuse_response)]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let referenced_message = match &ctx {
        Context::Prefix(prefix_ctx) => prefix_ctx.msg.message_reference.clone(),
        _ => None,
    };

    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
        let mut message = CreateMessage::default().content(content);
        
        if let Some(msg_id) = referenced_message {
            message = message.reference_message(msg_id);
        }
        
        ctx.channel_id()
            .send_message(ctx.serenity_context(), message)
            .await?;
    } else {
        ctx.send(
            CreateReply::default()
                .embed(create_error_embed(&format!("Tag `{}` does not exist", name)))
        ).await?;
    }

    Ok(())
}

/// Create a new tag
#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
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

/// Delete an existing tag
#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
async fn delete(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    match data.tag_db.delete_tag(&name, id).await {
        Ok(Some(fixed_name)) => {
            ctx.send(CreateReply::default().content(format!("Deleted tag `{}`", fixed_name))).await?
        }
        Ok(None) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&format!("Tag `{}` does not exist", name))))
                .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}

/// Edit an existing tag
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
        Ok(Some(fixed_name)) => {
            ctx.send(CreateReply::default().content(format!("Updated tag `{}`", fixed_name))).await?
        }
        Ok(None) => {
            ctx.send(
                CreateReply::default().embed(create_error_embed(&format!("Tag `{}` does not exist", name))),
            )
            .await?
        }
        Err(e) => {
            ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
        }
    };
    Ok(())
}

/// List all tags for this server
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

/// Preview a tag
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
            CreateReply::default().embed(create_error_embed(&format!("Tag `{}` does not exist", name))).ephemeral(true)
        ).await?;
    }
    Ok(())
}

/// View a tag in raw text
#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
async fn raw(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
        ctx.send(CreateReply::default().content(content.replace("`", "\\`"))).await?;
    } else {
        ctx.send(CreateReply::default().embed(create_error_embed(&format!("Tag `{}` does not exist", name))))
            .await?;
    }

    Ok(())
}

/// Create an alias for an existing tag
#[poise::command(prefix_command, slash_command, invoke_on_edit, reuse_response)]
async fn alias(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
    #[description = "Tag alias"] alias: String,
) -> Result<(), Error> {
    let (data, id) = get_data_and_id(ctx).await?;

    if let Ok(Some((_name, content))) = data.tag_db.get_tag(&name, id).await {
        match data.tag_db.create_tag(&alias, &content, id).await {
            Ok(_) => {
                ctx.send(CreateReply::default().content(format!("Created tag alias `{}`", alias)))
                    .await?
            }
            Err(e) => {
                ctx.send(CreateReply::default().embed(create_error_embed(&e.to_string()))).await?
            }
        };
    } else {
        ctx.send(CreateReply::default().embed(create_error_embed(&format!("Tag `{}` does not exist", name))))
            .await?;
    }
    Ok(())
}
