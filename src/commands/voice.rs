use std::fs::read_dir;
use std::path::{Path, PathBuf};
use rand::seq::SliceRandom;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, GuildId, ResolvedOption, ResolvedValue, User};
use serenity::prelude::{Context, TypeMapKey};
use rand::thread_rng;
use serenity::async_trait;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};
use songbird::events::TrackEvent;
use songbird::input::YoutubeDl;
use crate::HttpKey;

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

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


pub async fn join(ctx: &Context, guild_id: GuildId, user: &User) -> String {
    let guild = guild_id
        .to_guild_cached(&ctx.cache)
        .unwrap()
        .clone();
    let voice_channel_id = guild
        .voice_states
        .get(&user.id)
        .and_then(|voice_state| voice_state.channel_id);
    let voice_channel_id = match voice_channel_id {
        Some(v) => { v }
        None => {
            return "Ты должен быть в голосовом канале".to_string()
        }
    };
    let manager = songbird::get(ctx)
        .await
        .expect("Cannot register songbird manager")
        .clone();
    if let Ok(handler_lock) = manager.join(guild_id, voice_channel_id).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }
    "Я тут. Чё надо?".to_string()
}

pub async fn play(options: &[ResolvedOption<'_>], ctx: &Context, guild_id: GuildId) -> String {
    let url = options.first().expect("Haven't found any urls").clone().value;
    let url = match url {
        ResolvedValue::String(e) => {e.to_string()}
        _ => {"Nothing".to_string()}
    };
    let search = !url.starts_with("http");

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = if search {
            YoutubeDl::new_search(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };
        let _ = handler.play_input(src.clone().into());
        "Играем".to_string()
    } else {
        "Я не в канале".to_string()
    }
}

pub async fn phrase(ctx: &Context, guild_id: GuildId) -> String {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        // let src = get_music_file();
        let src = songbird::input::File::new(Path::new("/Users/mihailsmirnov/Downloads/Telegram Desktop/KINGPIN_rus/чуваки/ааа а все в порядке все в порядке.wav"));
        let _ = handler.play_input(src.clone().into());
        "Играем".to_string()
    } else {
        "Я не в канале".to_string()
    }
}

pub fn register_play() -> CreateCommand {
    CreateCommand::new("play")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "url",
            "Ссылка на видео"
        ).required(true)
        )
        .description("Проиграть с ютуба")
}

pub fn register_join() -> CreateCommand {
    CreateCommand::new("join")
        .description("Присоединиться к чату")
}

pub fn register_phrase() -> CreateCommand {
    CreateCommand::new("phrase")
        .description("Сказать рандомную фразу")
}