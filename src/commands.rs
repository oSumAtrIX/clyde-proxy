use poise::serenity_prelude::{CacheHttp, ChannelId, GuildId};

use crate::{Context, Error, CLYDE_ID};

/// Show this help menu.
#[poise::command(prefix_command, track_edits, slash_command)]
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
            extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
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

/// Pair Clyde from another Discord server with this server.
#[poise::command(slash_command, owners_only)]
pub async fn pair(
    ctx: Context<'_>,
    #[description = "The guild ID of the other server"] guild_id: String,
    #[description = "The channel ID of the other server"] channel_id: String,
) -> Result<(), Error> {
    let guild_id = GuildId(guild_id.parse()?);
    let channel_id = ChannelId(channel_id.parse()?);
    let to_channel_id = ctx.channel_id();

    let cache = ctx.cache().unwrap();

    let Some(guild) = cache.guild(guild_id) else {
		return Err("Guild not found".into());
	 };

    if cache.channel(channel_id.0).is_none() {
        return Err("Channel not found".into());
    };

    {
        let mut config = ctx.data().pair_config.lock().await;
        config.guild_id = guild_id;
        config.from_channel_id = channel_id;
        config.to_channel_id = to_channel_id;
    }

    ctx.defer_ephemeral().await?;

    ctx.say(format!(
        "Proxying <@{}> in channel <#{}> from guild {} to channel {}.",
        CLYDE_ID, channel_id, guild.name, to_channel_id.0
    ))
    .await?;
    Ok(())
}

/// Send a message to the paired server.
#[poise::command(slash_command, owners_only)]
pub async fn message(
    ctx: Context<'_>,
    #[description = "The message to send"] message: String,
) -> Result<(), Error> {
    let config = ctx.data().pair_config.lock().await;

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

    ctx.defer_ephemeral().await?;
    ctx.say("Sent message to paired server").await?;
    Ok(())
}
