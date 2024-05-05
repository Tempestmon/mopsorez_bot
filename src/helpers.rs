use rand::distributions::{Distribution, Uniform};
use serenity::all::{Context, Message};

pub async fn send_discord_message(ctx: &Context, msg: &Message, message: &str) {
    if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
        println!("Error sending message: {why:?}");
    }
    println!("Senging message: {message:#?}")
}

pub fn get_random_number(from: u16, to: u16) -> u16 {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(from..to + 1);
    die.sample(&mut rng)
}

pub fn is_answer_needed(prob_number: u16) -> bool {
    let throw = get_random_number(1, prob_number);
    if throw == prob_number {
        return true;
    }
    false
}