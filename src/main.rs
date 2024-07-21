use std::env;
use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client as HttpClient;
use serenity::all::{CreateMessage, Reaction};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::gateway::Ready;
use serenity::model::voice::VoiceState;
use serenity::prelude::*;
use songbird::driver::DecodeMode;
use songbird::{Config, SerenityInit};
use tokio::time::sleep;
use tracing::{error, info};

use commands::{delete, fisting, ping, rule34, voice};
use helpers::send_discord_message;

use crate::commands::voice::{join, play_file};

mod commands;
mod helpers;

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let author_name = &msg.author.name;
        let content = &msg.content;
        info!("{author_name} has sent a message {content:#?}");
        if !msg.author.bot && author_name != "tempestmon" {
            let _ = msg
                .react(ctx.clone().http, ReactionType::Unicode("🇬".to_owned()))
                .await;
            let _ = msg
                .react(ctx.clone().http, ReactionType::Unicode("🇦".to_owned()))
                .await;
            let _ = msg
                .react(ctx.clone().http, ReactionType::Unicode("🇾".to_owned()))
                .await;
            let _ = msg
                .react(ctx.clone().http, ReactionType::Unicode("🏳️‍🌈".to_owned()))
                .await;
            info!("Marking gay for {author_name}");
        }
        if !msg.author.bot {
            let bot_message = Self::get_answer_after_user_message(&msg).await;
            if let Some(message) = bot_message {
                send_discord_message(&ctx, &msg, message).await;
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let channel = reaction.channel_id;
        let user = reaction.member.unwrap();
        let reaction_text = reaction.emoji.clone().to_string();
        if reaction_text == "🏳️‍🌈" && user.user.name != "tempestmon" && !user.user.bot {
            let message =
                CreateMessage::new().content(format!("{user} поддержал LGBT {reaction_text}"));
            channel
                .send_message(&ctx.http, message)
                .await
                .expect("Could not send a message");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} has connected!", ready.user.name);

        let guild = ready.guilds[0];
        let guild_id = guild.id;

        guild_id
            .set_commands(
                &ctx.http,
                vec![
                    ping::register(),
                    rule34::register(),
                    voice::register_play(),
                    voice::register_join(),
                    voice::register_phrase(),
                    fisting::register_fisting(),
                    fisting::register_defense(),
                    delete::register(),
                ],
            )
            .await
            .expect("failed to create application command");
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        let cache = &ctx.cache;
        if new_state.channel_id.is_none() {
            let main_channel = &ctx
                .cache
                .channel(old_state.clone().unwrap().channel_id.unwrap())
                .unwrap()
                .clone();
            let old_user = old_state.unwrap().member.unwrap();
            let message_builder =
                CreateMessage::new().content(format!("{old_user}, хули ты вышел?"));
            main_channel
                .send_message(&ctx.http, message_builder)
                .await
                .expect("Could not send a message");
            return;
        }
        let new_channel = new_state
            .channel_id
            .expect("No ChannelId for new channel")
            .to_channel_cached(cache)
            .expect("Cannot get cached channel")
            .clone();
        let new_channel_name = &new_channel.name;
        let members = new_channel
            .members(cache)
            .expect("Couldn't get members in new channel");
        let new_user = new_state
            .user_id
            .to_user(&ctx.http)
            .await
            .expect("Couldn't get user by userid");
        let new_username = new_user.name;
        info!("User {new_username} state updated in channel {new_channel_name}");
        if new_user.bot {
            return;
        }
        let new_members_count = members.len();
        let state = new_state.clone();
        if state.self_mute {
            play_file(
                &ctx,
                new_channel.guild_id,
                PathBuf::from(
                    env::var("OTVET")
                        .expect("Couldn't find OTVET variable")
                        .to_string(),
                ),
            )
            .await;
        }

        match old_state {
            None => {
                join(&ctx, new_state.guild_id.unwrap(), &new_state.user_id).await;
                sleep(Duration::new(1, 0)).await;
                play_file(
                    &ctx,
                    new_channel.guild_id,
                    PathBuf::from(
                        env::var("HOOLI")
                            .expect("Couldn't find HOOLI variable")
                            .to_string(),
                    ),
                )
                .await;
            }
            Some(old_channel) => {
                let channel = old_channel
                    .channel_id
                    .unwrap()
                    .to_channel_cached(cache)
                    .unwrap()
                    .clone();
                let old_channel_name = &channel.name;
                let old_members = channel.members(cache).unwrap();
                let old_members_count = old_members.len();
                info!("User state updated from {old_channel_name} to {new_channel_name}");
                if old_members_count <= new_members_count {
                    sleep(Duration::new(2, 0)).await;
                    play_file(
                        &ctx,
                        new_channel.guild_id,
                        PathBuf::from(
                            env::var("PNH")
                                .expect("Couldn't find PNH variable")
                                .to_string(),
                        ),
                    )
                    .await;
                    if old_members_count <= 1 {
                        join(&ctx, old_channel.guild_id.unwrap(), &old_channel.user_id).await;
                    }
                }
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let channel_id = command.channel_id;
            let guild_id = command.data.guild_id.expect("No guild was found");
            let command_options = &command.data.options();

            let content = match command.data.name.as_str() {
                "ping" => Some(ping::run()),
                "rule34" => Some(rule34::find_image(command_options).await),
                "join" => Some(join(&ctx.clone(), guild_id, &command.user.id).await),
                "play" => Some(voice::play(command_options, &ctx.clone(), guild_id).await),
                "phrase" => Some(voice::play_random_file(&ctx.clone(), guild_id).await),
                "fisting" => Some(fisting::perform_fisting(command_options, &command.user).await),
                "delete" => Some(
                    delete::delete_messages(command_options, &ctx.clone(), guild_id, &channel_id)
                        .await,
                ),
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

impl Handler {
    async fn get_answer_after_user_message(msg: &Message) -> Option<&str> {
        let bot_message = match msg.content.as_str() {
            "да" | "Да" | "Da" | "da" => Some("Пидора слова"),
            "нет" | "Нет" | "Net" | "net" => Some("Пидора ответ"),
            "Мопсы пидоры?" | "Мопсы чурки?" => Some("Да"),
            _ => match msg.author.name.as_str() {
                "_fatpug_" => match helpers::is_answer_needed(3) {
                    true => Some("Заткнись, мопс"),
                    false => None,
                },
                _ => match helpers::is_answer_needed(6) {
                    true => Some("Помолчи, заебал"),
                    false => None,
                },
            },
        };
        bot_message
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt().with_target(false).finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber.");

    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    let songbird_config = Config::default().decode_mode(DecodeMode::Decode);
    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(Handler)
        .register_songbird_from_config(songbird_config)
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
