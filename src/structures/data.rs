pub struct PrefixMap;

use dashmap::DashMap;
use serenity::client::bridge::voice::ClientVoiceManager;
use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::{GuildId, UserId},
    prelude::{Mutex, TypeMapKey},
};
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};

impl TypeMapKey for PrefixMap {
    type Value = Arc<DashMap<GuildId, String>>;
}
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ConnectionPool;

impl TypeMapKey for ConnectionPool {
    type Value = PgPool;
}
pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}
