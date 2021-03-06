use std::sync::Arc;

use crate::structures::data::{ConnectionPool, PrefixMap};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use std::env;

#[command]
#[required_permissions("ADMINISTRATOR")]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (pool, prefixes, default_prefix) = {
        let data = ctx.data.read().await;

        let pool = data.get::<ConnectionPool>().cloned().unwrap();
        let prefixes = data.get::<PrefixMap>().unwrap().clone();
        let default_prefix = env::var("DEFAULT_PREFIX").expect("problem getting default prefix");

        (pool, prefixes, default_prefix)
    };
    let guild_id = msg.guild_id.unwrap();
    let guild_name = msg.guild(ctx).await.unwrap().name;

    if args.is_empty() {
        let cur_prefix = match prefixes.get(&guild_id) {
            Some(prefix_guard) => prefix_guard.value().to_owned(),
            None => default_prefix,
        };

        msg.channel_id
            .say(
                ctx,
                format!("My prefix for `{}` is `{}`", guild_name, cur_prefix),
            )
            .await?;
        return Ok(());
    }

    let new_prefix = args.single::<String>().unwrap();

    if new_prefix == default_prefix {
        sqlx::query!(
            "UPDATE guild_info SET prefix = null WHERE guild_id = $1",
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        prefixes.remove(&guild_id);
    } else {
        sqlx::query!(
            "UPDATE guild_info SET prefix = $1 WHERE guild_id = $2",
            new_prefix,
            guild_id.0 as i64
        )
        .execute(&pool)
        .await?;

        prefixes.insert(guild_id, new_prefix.to_owned());
    }

    msg.channel_id
        .say(
            ctx,
            format!("My new prefix is `{}` for `{}`!", new_prefix, guild_name),
        )
        .await?;

    Ok(())
}
