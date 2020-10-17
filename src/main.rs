use std::{collections::HashSet, env, sync::Arc};

use serenity::model::channel::Message;
use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::group, standard::macros::hook, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use sqlx::PgPool;
use tokio::time::{delay_for, Duration};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::meta::*;

use crate::commands::config::*;
use crate::helpers::*;
use crate::structures::data::{ConnectionPool, PrefixMap};
mod commands;
mod helpers;
mod moderation;
mod structures;
use crate::moderation::purge::*;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

//TODO Ban command, kick command

//TODO Mute timers (how to handle bot restarts?) and mute role in database
//TODO tool to make database if none is available
//TODO help command(s)
//TODO reminders
//TODO maybe a scheduler or timer object? (storing object on disk via .RON?)
//TODO invite command (ez)

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            // Note that array index 0 is 0-indexed, while index 1 is 1-indexed.
            //
            // This may seem unintuitive, but it models Discord's behaviour.
            println!(
                "{} is connected on shard {}/{}!",
                ready.user.name, shard[0], shard[1],
            );
        }
    }
}

#[group]
#[commands(ping, die, prefix, purge)]
struct General;

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load env file");
    //this next boi logs the env variables.
    //and yes, this portion of the code is essentially the serenity example part
    //i don't see any reason to do it differently seeing as this is exactly what i need
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let http = Http::new_with_token(&token);
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    #[hook]
    async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let (prefixes, default_prefix) = {
            let data = ctx.data.read().await;
            let prefixes = data.get::<PrefixMap>().cloned().unwrap();
            let default_prefix =
                env::var("DEFAULT_PREFIX").expect("problem getting default prefix");

            (prefixes, default_prefix)
        };
        let guild_id = msg.guild_id.unwrap();
        match prefixes.get(&guild_id) {
            Some(prefix_guard) => Some(prefix_guard.value().to_owned()),
            None => Some(default_prefix),
        }
    }
    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(dynamic_prefix)
                .on_mention(Some(_bot_id))
        })
        .group(&GENERAL_GROUP);
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // let manager = client.shard_manager.clone();
    // tokio::spawn(async move {
    // loop {
    // delay_for(Duration::from_secs(30)).await;
    // let lock = manager.lock().await;
    // let shard_runners = lock.runners.lock().await;
    // These commented lines are potentially useful, but I do not need them right now.
    // for (id, runner) in shard_runners.iter() {
    // println!(
    //        "Shard ID {} is {} with a latency of {:?}",
    //       id, runner.stage, runner.latency,
    //   );
    //}
    //}
    //});
    let pool = db::get_db_pool(env::var("DATABASE_URL").expect("define a database url in env"))
        .await
        .unwrap();
    let prefixes = db::fetch_prefixes(&pool).await.unwrap();

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<ConnectionPool>(pool);
        data.insert::<PrefixMap>(Arc::new(prefixes));
    }
    if let Err(why) = client.start_shards(2).await {
        error!("Client error: {:?}", why);
    }
}
