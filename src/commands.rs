use poise::serenity_prelude::{CacheHttp};

use crate::{Context, Error, ProxyConfiguration, CLYDE_ID};

/// Show this help menu.
#[poise::command(prefix_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Proxy Clyde from one Discord server to another.",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Proxy Clyde from another Discord server to your server.
#[poise::command(prefix_command, owners_only)]
pub async fn proxy(
    ctx: Context<'_>,
    #[description = "The channel ID of the other server"] channel_id: String,
) -> Result<(), Error> {
    let Ok(channel) = ctx.http().get_channel(channel_id.parse()?).await else {
        return Err("Can not find channel ID".into());
    };

    let config = ProxyConfiguration {
        from_channel_id: ctx.channel_id(),
        to_channel_id: channel.id(),
    };

    *ctx.data().proxy_config.lock().await = Some(config);

    ctx.say(format!(
        "Proxying <@{}> from channel <#{}>.",
        CLYDE_ID, channel_id
    ))
    .await?;

    Ok(())
}

/// Send a message to the proxied server.
#[poise::command(prefix_command)]
pub async fn message(
    ctx: Context<'_>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
    let Some(ref mut config) = *ctx.data().proxy_config.lock().await 
        else { return Err("No proxy configured".into()); };

    config.from_channel_id = ctx.channel_id();

    config
        .to_channel_id
        .say(
            ctx.http(),
            format!(
                "<@{}> Hello, my name is {}. {}",
                CLYDE_ID,
                ctx.author().name,
                message
            ),
        )
        .await?;
    Ok(())
}