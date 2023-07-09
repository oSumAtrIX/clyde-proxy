use poise::serenity_prelude::{CacheHttp, ChannelId, GuildId};

use crate::{Context, Error, CLYDE_ID};

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
            extra_text_at_bottom: "Proxy Clyde from one server to another",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// Register slash commands.
#[poise::command(prefix_command, owners_only)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Proxy Clyde from another Discord server to your server.
#[poise::command(prefix_command, owners_only)]
pub async fn proxy(
    ctx: Context<'_>,
    #[description = "The guild ID of the other server"] guild_id: String,
    #[description = "The channel ID of the other server"] channel_id: String,
) -> Result<(), Error> {
    let guild_id = GuildId(guild_id.parse()?);
    let channel_id = ChannelId(channel_id.parse()?);

    let http = ctx.http();

    let Ok(guild) = http.get_guild(guild_id.0).await else {
		return Err("Guild not found".into());
	 };

    if http.get_channel(channel_id.0).await.is_err() {
        return Err("Channel not found".into());
    };

    {
        let mut config = ctx.data().pair_config.lock().await;
        config.guild_id = guild_id;
        config.from_channel_id = channel_id;
    }

    ctx.say(format!(
        "Proxying <@{}> in channel <#{}> from guild \"{}\".",
        CLYDE_ID, channel_id, guild.name
    ))
    .await?;
    Ok(())
}

/// Send a message to the paired server.
#[poise::command(prefix_command)]
pub async fn message(
    ctx: Context<'_>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
    let mut config = ctx.data().pair_config.lock().await;

    config.to_channel_id = ctx.channel_id();

    config
        .from_channel_id
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
