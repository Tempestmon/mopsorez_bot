use std::env;
use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use rand::distributions::{Distribution, Uniform};

struct Handler;

async fn is_answer_needed(prob_number: i8) -> bool {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..prob_number + 1);
    let throw = die.sample(&mut rng);
    if throw == prob_number {
        return true
    }
    false
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.author.bot {
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
            match bot_message {
                Some(message) => {
                    if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                        println!("Error sending message: {why:?}");
                    }
                }
                None => {}
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

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