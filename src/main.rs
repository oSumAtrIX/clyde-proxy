#![warn(clippy::str_to_string)]

mod commands;

use poise::{
    futures_util::lock::Mutex,
    serenity_prelude::{self as serenity, ChannelId, GuildId, UserId},
};
use std::env::var;
use tokio::sync::RwLock;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Default)]
pub struct Data {
    pair_config: Mutex<PairConfiguration>,
}

#[derive(Default)]
pub struct PairConfiguration {
    guild_id: GuildId,
    from_channel_id: ChannelId,
    to_channel_id: ChannelId,
}

#[derive(Default)]
struct Handler {
    options: poise::FrameworkOptions<Data, Error>,
    data: Data,
    bot_id: RwLock<Option<UserId>>,
    shard_manager:
        std::sync::Mutex<Option<std::sync::Arc<tokio::sync::Mutex<serenity::ShardManager>>>>,
}

const CLYDE_ID: u64 = 1081004946872352958;

// Custom handler to dispatch poise events.
impl Handler {
    pub fn new(options: poise::FrameworkOptions<Data, Error>) -> Self {
        Self {
            options,
            ..Default::default()
        }
    }

    async fn dispatch_poise_event(&self, ctx: &serenity::Context, event: &poise::Event<'_>) {
        let framework_data = poise::FrameworkContext {
            bot_id: self.bot_id.read().await.unwrap(),
            options: &self.options,
            user_data: &self.data,
            shard_manager: &(*self.shard_manager.lock().unwrap()).clone().unwrap(),
        };

        poise::dispatch_event(framework_data, ctx, event).await;
    }
}

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn message(&self, ctx: serenity::Context, new_message: serenity::Message) {
        if new_message.author.bot && new_message.author.id == CLYDE_ID {
            let data = self.data.pair_config.lock().await;

            if data.to_channel_id == new_message.channel_id {
                let _ = data
                    .to_channel_id
                    .say(&ctx.http, &new_message.content)
                    .await;
            }
        }

        self.dispatch_poise_event(&ctx, &poise::Event::Message { new_message })
            .await;
    }

    async fn interaction_create(&self, ctx: serenity::Context, interaction: serenity::Interaction) {
        self.dispatch_poise_event(&ctx, &poise::Event::InteractionCreate { interaction })
            .await;
    }

    async fn ready(&self, _ctx: serenity::Context, ready: serenity::Ready) {
        *self.bot_id.write().await = Some(ready.user.id);
    }
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv::dotenv().expect("Failed to read .env file");

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help(),
            commands::register(),
            commands::pair(),
            commands::message(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            mention_as_prefix: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}", ctx.command().qualified_name);
            })
        },
        owners: vec![UserId(
            var("OWNER_ID")
                .expect("Missing `OWNER_ID` environment variable")
                .parse::<u64>()
                .expect("Failed to parse `OWNER_ID` environment variable"),
        )]
        .into_iter()
        .collect(),
        skip_checks_for_owners: false,
        ..Default::default()
    };

    let handler = std::sync::Arc::new(Handler::new(options));

    let mut client = serenity::Client::builder(
        var("DISCORD_TOKEN").expect("Missing `DISCORD_TOKEN` environment variable"),
        serenity::GatewayIntents::non_privileged()
            | serenity::GatewayIntents::MESSAGE_CONTENT
            | serenity::GatewayIntents::GUILD_MESSAGES,
    )
    .event_handler_arc(handler.clone())
    .await?;

    *handler.shard_manager.lock().unwrap() = Some(client.shard_manager.clone());
    client.start().await?;

    Ok(())
}
