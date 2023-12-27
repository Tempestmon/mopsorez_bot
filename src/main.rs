use std::env;
use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};
use serde_json::Error;

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

#[derive(Deserialize, Serialize, Debug)]
struct Rule34Model {
    file_url: String,
}


#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.contains("!rule34") {
            let message_split: Vec<&str> = msg.content.as_str().split(" ").collect();
            if message_split.len() > 1 {
                let search_tag = message_split.get(1).expect("Could get shit");
                let request = reqwest::get(format!("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index&json=1&limit=50&tags={search_tag}")).await.expect("Error trying to call rule34");
                let image = request.text().await.expect("Got no image");
                let model: Result<Vec<Rule34Model>, Error> = serde_json::from_str(image.as_str());
                match model {
                    Ok(model) => {
                        let random = get_random_number(model.len() as i8).await - 1;
                        println!("{model:#?}");
                        let url = model.get(random as usize).expect("No model was found").file_url.clone();
                        Self::send_discord_message(&ctx, &msg, url.as_str()).await;
                    }
                    Err(_) => {
                        Self::send_discord_message(&ctx, &msg, "Я такую хуйню найти не могу").await;
                    }
                }
            } else {
                Self::send_discord_message(&ctx, &msg, "Введи тег для поиска, придурок").await
            }
        }
        if !msg.author.bot {
            let bot_message = Self::get_answer_after_user_message(&msg).await;
            match bot_message {
                Some(message) => {
                    Self::send_discord_message(&ctx, &msg, message).await;
                }
                None => {}
            }
        }
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

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

    async fn send_discord_message(ctx: &Context, msg: &Message, message: &str) {
        if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
            println!("Error sending message: {why:?}");
        }
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