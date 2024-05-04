use serenity::all::{ChannelId, Context, Message};

pub async fn send_discord_message(ctx: &Context, msg: &Message, message: &str) {
    if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
        println!("Error sending message: {why:?}");
    }
    println!("Senging message: {message:#?}")
}

pub async fn get_channel_users(channel: ChannelId) {
    todo!()
}
