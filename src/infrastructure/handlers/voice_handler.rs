use std::time::Duration;

use serenity::all::{Context, CreateMessage, VoiceState};
use tokio::time::sleep;
use tracing::{error, info};

use crate::commands::voice::{join, play_file};
use crate::config::Config;
use crate::error::BotError;

pub async fn handle_voice_state_update(
    ctx: &Context,
    old_state: Option<VoiceState>,
    new_state: VoiceState,
) -> Result<(), BotError> {
    // User left all voice channels
    if new_state.channel_id.is_none() {
        let Some(old_st) = old_state else { return Ok(()) };
        let Some(old_channel_id) = old_st.channel_id else { return Ok(()) };
        let Some(old_guild_id) = old_st.guild_id else { return Ok(()) };
        let Some(old_member) = old_st.member else { return Ok(()) };
        if old_member.user.bot {
            return Ok(());
        }
        let channel_opt = {
            ctx.cache
                .guild(old_guild_id)
                .and_then(|g| g.channels.get(&old_channel_id).cloned())
        };
        let Some(main_channel) = channel_opt else {
            error!("Channel {old_channel_id} not found in cache");
            return Ok(());
        };
        main_channel
            .send_message(
                &ctx.http,
                CreateMessage::new().content(format!("{old_member}, хули ты вышел?")),
            )
            .await?;
        return Ok(());
    }

    let new_channel_id = new_state.channel_id.unwrap(); // safe: checked above
    let Some(guild_id) = new_state.guild_id else { return Ok(()) };
    let new_user_id = new_state.user_id;
    let self_mute = new_state.self_mute;

    // Extract what we need before any .await — Ref must not cross await points
    let (new_channel_name, new_members_count) = {
        let Some(new_channel) = ctx
            .cache
            .guild(guild_id)
            .and_then(|g| g.channels.get(&new_channel_id).cloned())
        else {
            error!("Channel {new_channel_id} not in cache");
            return Ok(());
        };
        (
            new_channel.name.clone(),
            new_channel.members(&ctx.cache).unwrap_or_default().len(),
        )
    };

    let new_user = match new_user_id.to_user(&ctx.http).await {
        Ok(u) => u,
        Err(e) => {
            error!("Couldn't get user: {e}");
            return Ok(());
        }
    };

    if new_user.bot {
        return Ok(());
    }
    info!("User {} state updated in channel {new_channel_name}", new_user.name);

    let (otvet_path, hooli_path, pnh_path) = {
        let data = ctx.data.read().await;
        let cfg = data.get::<Config>().expect("Config must be in TypeMap");
        (cfg.otvet.clone(), cfg.hooli.clone(), cfg.pnh.clone())
    };

    if self_mute {
        play_file(ctx, guild_id, otvet_path).await;
    }

    match old_state {
        None => {
            join(ctx, guild_id, &new_user_id).await;
            sleep(Duration::new(1, 0)).await;
            play_file(ctx, guild_id, hooli_path).await;
        }
        Some(old_st) => {
            let Some(old_channel_id) = old_st.channel_id else { return Ok(()) };
            let (old_channel_name, old_members_count) = {
                let Some(old_channel) = ctx
                    .cache
                    .guild(guild_id)
                    .and_then(|g| g.channels.get(&old_channel_id).cloned())
                else {
                    error!("Old channel {old_channel_id} not in cache");
                    return Ok(());
                };
                (
                    old_channel.name.clone(),
                    old_channel.members(&ctx.cache).unwrap_or_default().len(),
                )
            };
            info!("User state updated from {old_channel_name} to {new_channel_name}");
            if old_members_count <= new_members_count {
                sleep(Duration::new(2, 0)).await;
                play_file(ctx, guild_id, pnh_path).await;
                if old_members_count <= 1 {
                    join(ctx, guild_id, &old_st.user_id).await;
                }
            }
        }
    }

    Ok(())
}
