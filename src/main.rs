mod commands;
mod helpers;

use commands::{delete, ping, rule34, voice};
use helpers::send_discord_message;
use rand::distributions::{Distribution, Uniform};
use reqwest::Client as HttpClient;
use serenity::all::ReactionType;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use songbird::SerenityInit;
use std::env;

struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct Handler;

fn get_random_number(number: i8) -> i8 {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..number + 1);
    die.sample(&mut rng)
}

fn is_answer_needed(prob_number: i8) -> bool {
    let throw = get_random_number(prob_number);
    if throw == prob_number {
        return true;
    }
    false
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot && msg.author.name != "tempestmon" {
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
            println!("Marking gay");
        }
        if !msg.author.bot {
            let bot_message = Self::get_answer_after_user_message(&msg).await;
            match bot_message {
                Some(message) => {
                    send_discord_message(&ctx, &msg, message).await;
                }
                None => {}
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} has connected!", ready.user.name);

        let guild = ready.guilds[0];
        assert_eq!(guild.unavailable, true);
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
                    delete::register(),
                ],
            )
            .await
            .expect("failed to create application command");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let channel_id = command.channel_id;
            let guild_id = command.data.guild_id.unwrap();
            let command_options = &command.data.options();

            let content = match command.data.name.as_str() {
                "ping" => Some(ping::run()),
                "rule34" => Some(rule34::find_image(command_options).await),
                "join" => Some(
                    voice::join(&ctx.clone(), command.data.guild_id.unwrap(), &command.user).await,
                ),
                "play" => Some(voice::play(command_options, &ctx.clone(), guild_id).await),
                "phrase" => Some(voice::phrase(&ctx.clone(), guild_id).await),
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
                    println!("Cannot respond to slash command: {why}");
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
                "_fatpug_" => match is_answer_needed(3) {
                    true => Some("Заткнись, мопс"),
                    false => None,
                },
                _ => match is_answer_needed(6) {
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
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::builder(&token, GatewayIntents::all())
        .event_handler(Handler)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Err creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
