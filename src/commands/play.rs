use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::Arc;
use rand::seq::SliceRandom;
use serenity::all::{CreateCommand, GuildId, User};
use serenity::prelude::Context;
use rand::thread_rng;
use songbird::{Driver, Songbird, TrackEvent};
use songbird::input::File;
use songbird::tracks::Track;

fn get_music_file() -> PathBuf {
    let music_directory = "~/Downloads/Telegram Desktop/KINGPIN_rus/чуваки";
    let mut music_files: Vec<_> = read_dir(music_directory)
        .expect("Failed to read music directory")
        .map(|entry| entry.unwrap().path())
        .collect();
    let mut rng = thread_rng();
    music_files.shuffle(&mut rng);
    let random_file = music_files[0].clone();
    random_file
}


pub async fn play(ctx: &Context, guild_id: &GuildId, user: &User) -> String {
    let guild = guild_id
        .to_guild_cached(&ctx.cache)
        .unwrap();
    let channel_id = guild
        .voice_states
        .get(&user.id)
        .and_then(|voice_state| voice_state.channel_id);
    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            return String::from("Pidor!!!");
        },
    };
    // let manager = songbird::get(ctx)
    //     .await
    //     .expect("Pizdec menya pryet")
    //     .clone();
    String::from("Cyka!!")
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play")
        .description("Сказать прикол")
}