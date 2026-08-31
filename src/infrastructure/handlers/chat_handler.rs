use serenity::all::{Context, CreateMessage, Message, Reaction, ReactionType};
use tracing::{error, info};

use crate::config::Config;
use crate::helpers;

pub async fn handle_message(ctx: &Context, msg: Message) {
    let author_name = &msg.author.name;
    info!("{author_name} has sent a message {:?}", msg.content);

    let bot_owner = {
        let data = ctx.data.read().await;
        data.get::<Config>()
            .expect("Config must be in TypeMap")
            .bot_owner
            .clone()
    };

    if !msg.author.bot && author_name != &bot_owner {
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
        if let Some(reply) = get_auto_reply(&msg) {
            helpers::send_discord_message(ctx, &msg, reply).await;
        }
    }
}

pub async fn handle_reaction_add(ctx: &Context, reaction: Reaction) {
    let Some(user) = reaction.member else { return };
    let reaction_text = reaction.emoji.to_string();

    let bot_owner = {
        let data = ctx.data.read().await;
        data.get::<Config>()
            .expect("Config must be in TypeMap")
            .bot_owner
            .clone()
    };

    if reaction_text == "🏳️‍🌈" && user.user.name != bot_owner && !user.user.bot {
        let message =
            CreateMessage::new().content(format!("{user} поддержал LGBT {reaction_text}"));
        if let Err(e) = reaction.channel_id.send_message(&ctx.http, message).await {
            error!("Could not send reaction message: {e}");
        }
    }
}

fn get_auto_reply(msg: &Message) -> Option<&'static str> {
    match msg.content.as_str() {
        "да" | "Да" | "Da" | "da" => Some("Пидора слова"),
        "нет" | "Нет" | "Net" | "net" => Some("Пидора ответ"),
        "Мопсы пидоры?" | "Мопсы чурки?" => Some("Да"),
        _ => match msg.author.name.as_str() {
            "_fatpug_" => helpers::is_answer_needed(3).then_some("Заткнись, мопс"),
            _ => helpers::is_answer_needed(6).then_some("Помолчи, заебал"),
        },
    }
}
