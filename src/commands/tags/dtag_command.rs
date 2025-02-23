use poise::CreateReply;

use crate::{commands::{tag_utils::get_data_and_id, utils::create_error_embed}, Context, Error};

#[poise::command(prefix_command)]
pub async fn dtag(
    ctx: Context<'_>,
    #[description = "Tag name"] name: String,
) -> Result<(), Error> {
    let msg = match &ctx {
        Context::Prefix(prefix_ctx) => prefix_ctx.msg,
        _ => panic!("dtag may only be used with a prefix command"),
    };

    let is_reply = msg.referenced_message.is_some();

    let (data, id) = get_data_and_id(ctx).await?;
    
    if let Ok(Some(content)) = data.tag_db.get_tag(&name, id).await {
        let builder = if is_reply {
            CreateReply::default().content(content).reply(true)
        } else {
            CreateReply::default().content(content)
        };

        ctx.send(builder).await?;
    } else {
        ctx.send(CreateReply::default().embed(create_error_embed("Tag not found")))
            .await?;
    }
    msg.delete(&ctx.serenity_context()).await?;
    Ok(())
}