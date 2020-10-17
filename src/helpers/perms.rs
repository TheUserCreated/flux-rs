use crate::ConnectionPool;
use serenity::{model::prelude::*, prelude::*};

pub async fn check_admin(
    ctx: &Context,
    msg: &Message,
    user_id: Option<UserId>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
    let perms = match channel
        .permissions_for_user(ctx, user_id.unwrap_or(msg.author.id))
        .await
    {
        Ok(perms) => perms,
        Err(_) => return Ok(false),
    };

    if perms.administrator() {
        Ok(true)
    } else {
        Ok(false)
    }
}

//pub async fn check_mod(ctx: &Context, msg: &Message, user_id: Option<UserId>) -> Result<bool> {
//    let channel = msg.channel(ctx).await.unwrap().guild().unwrap();
//    let is_admin = channel
//        .permissions_for_user(ctx, user_id.unwrap_or(msg.author.id))
//        .await?
//        .administrator();
//    if is_admin {
//        return Ok(true);
//    }
//}
