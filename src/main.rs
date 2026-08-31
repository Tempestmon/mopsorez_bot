use std::sync::Arc;

use reqwest::Client as HttpClient;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::event::VoiceServerUpdateEvent;
use serenity::model::gateway::Ready;
use serenity::model::voice::VoiceState;
use serenity::all::Reaction;
use serenity::prelude::*;
use songbird::driver::DecodeMode;
use songbird::{Config as SongbirdConfig, SerenityInit};
use tracing::{error, info};

use commands::{delete, fisting, ping, rule34, voice};
use config::Config;
use error::BotError;
use infrastructure::fisting_repository::{FistingRepoKey, JsonFistingRepository};
use infrastructure::handlers::{chat_handler, voice_handler};

use crate::commands::voice::join;

mod commands;
mod config;
mod domain;
mod error;
mod helpers;
mod infrastructure;

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        chat_handler::handle_message(&ctx, msg).await;
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        chat_handler::handle_reaction_add(&ctx, reaction).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} has connected!", ready.user.name);
        let guild_id = ready.guilds[0].id;
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

    async fn voice_server_update(&self, _ctx: Context, update: VoiceServerUpdateEvent) {
        info!("Voice server endpoint assigned: {}", update.endpoint.as_deref().unwrap_or("none"));
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        old_state: Option<VoiceState>,
        new_state: VoiceState,
    ) {
        if let Err(e) = voice_handler::handle_voice_state_update(&ctx, old_state, new_state).await
        {
            error!("voice_state_update error: {e}");
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else { return };

        let channel_id = command.channel_id;
        let guild_id = command.data.guild_id.expect("No guild was found");
        let command_options = &command.data.options();

        // Clone repo Arc out of TypeMap before any .await
        let fisting_repo = {
            let data = ctx.data.read().await;
            data.get::<FistingRepoKey>()
                .expect("FistingRepo must be in TypeMap")
                .clone()
        };

        let result: Result<String, BotError> = match command.data.name.as_str() {
            "ping" => Ok(ping::run()),
            "rule34" => rule34::find_image(command_options).await,
            "join" => Ok(join(&ctx, guild_id, &command.user.id).await),
            "play" => Ok(voice::play(command_options, &ctx, guild_id).await),
            "phrase" => Ok(voice::play_random_file(&ctx, guild_id).await),
            "fisting" => {
                fisting::perform_fisting(command_options, &command.user, &*fisting_repo).await
            }
            "fisting_defense" => {
                fisting::defend_from_fisting(&command.user, &*fisting_repo).await
            }
            "delete" => {
                delete::delete_messages(command_options, &ctx, guild_id, &channel_id).await
            }
            _ => Ok("not implemented :(".to_string()),
        };

        let content = match result {
            Ok(s) => s,
            Err(e) => {
                error!("Command '{}' failed: {e}", command.data.name);
                "Что-то пошло не так".to_string()
            }
        };

        let data = CreateInteractionResponseMessage::new().content(content);
        let builder = CreateInteractionResponse::Message(data);
        if let Err(why) = command.create_response(&ctx.http, builder).await {
            error!("Cannot respond to slash command: {why}");
        }
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber.");

    let bot_config = Config::from_env();
    let token = bot_config.discord_token.clone();
    let fisting_data_path = bot_config.fisting_data.clone();

    let fisting_repo: Arc<dyn infrastructure::fisting_repository::FistingRepository> =
        Arc::new(JsonFistingRepository::new(fisting_data_path));

    let songbird_config = SongbirdConfig::default().decode_mode(DecodeMode::Decode);
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .register_songbird_from_config(songbird_config)
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<Config>(bot_config)
        .type_map_insert::<FistingRepoKey>(fisting_repo)
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
