mod ping;
mod rule34;
mod helpers;

use std::env;
use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use rand::distributions::{Distribution, Uniform};
use helpers::send_discord_message;
use rule34::find_image;

struct Handler;

async fn get_random_number(number: i8) -> i8 {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..number + 1);
    die.sample(&mut rng)
}

async fn is_answer_needed(prob_number: i8) -> bool {
    let throw = get_random_number(prob_number).await;
    if throw == prob_number {
        return true
    }
    false
}




#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.contains("!rule34") {
            find_image(&ctx, &msg).await;
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

        guild_id.set_commands(&ctx.http, vec![
            ping::register(),
        ])
            .await
            .expect("failed to create application command");
    }
}

impl Handler {
    async fn get_answer_after_user_message(msg: &Message) -> Option<&str> {
        let bot_message = match msg.content.as_str() {
            "да" | "Да" | "Da" | "da" => {
                Some("Пидора слова")
            }
            "нет" | "Нет" | "Net" | "net" => {
                Some("Пидора ответ")
            }
            "Мопсы пидоры?" | "Мопсы чурки?" => {
                Some("Да")
            }
            _ => {
                match msg.author.name.as_str() {
                    "_fatpug_" => {
                        match is_answer_needed(3).await {
                            true => {
                                Some("Заткнись, мопс")
                            }
                            false => {
                                None
                            }
                        }
                    }
                    _ => {
                        match is_answer_needed(6).await {
                            true => {
                                Some("Помолчи, заебал")
                            }
                            false => {
                                None
                            }
                        }
                    }
                }
            }
        };
        bot_message
    }
}


#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client =
        Client::builder(&token, GatewayIntents::all())
            .event_handler(Handler)
            .await
            .expect("Err creating client");
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}