use crate::helpers::perms;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

#[command]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if !perms::check_admin(ctx, msg, None).await? {
        msg.reply(ctx, "You lack the perms to use this command")
            .await?;
        return Ok(());
    }

    if args.is_empty() {
        msg.reply(ctx, "I need a number of messages to purge")
            .await?;
        return Ok(());
    }

    let num = match args.single::<u64>() {
        Ok(num) => num,
        Err(_) => {
            msg.reply(ctx, "I don't think that's a real number.")
                .await?;
            return Ok(());
        }
    };

    let id = ctx.http.get_message(msg.channel_id.0, num).await.is_ok();

    let mut messages: Vec<Message> = Vec::new();

    if id {
        let start_id = MessageId::from(num);
        messages = msg.channel_id.messages(ctx, |m| m.after(start_id)).await?;
    } else {
        if num > 100 {
            msg.reply(ctx, "You can't clear more than 100 messages at once")
                .await?;

            return Ok(());
        }
        messages = msg.channel_id.messages(ctx, |m| m.limit(num + 1)).await?;
    }

    match msg
        .channel_id
        .delete_messages(ctx, messages.into_iter().map(|m| m.id))
        .await
    {
        Ok(_) => {
            msg.channel_id.say(ctx, "Messages cleared.").await?;
        }
        Err(error) => {
            msg.channel_id
                .say(ctx, "I can't delete messages older than 2 weeks.")
                .await?;
            eprintln!(
                "Command purge errored in guild {}: {}",
                msg.guild_id.unwrap().0,
                error
            );
        }
    };

    Ok(())
}
