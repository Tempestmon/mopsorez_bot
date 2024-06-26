use std::env;
use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use dashmap::DashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, GuildId, ResolvedOption, ResolvedValue,
    UserId,
};
use serenity::async_trait;
use serenity::prelude::Context;
use songbird::events::TrackEvent;
use songbird::input::{File, Input, YoutubeDl};
use songbird::{CoreEvent, Event, EventContext, EventHandler as VoiceEventHandler};
use tracing::{error, info, warn};

use crate::{helpers, HttpKey};

pub(crate) struct TrackErrorNotifier;

#[derive(Clone, Debug)]
struct Receiver {
    inner: Arc<InnerReceiver>,
    guild_id: Option<GuildId>,
    ctx: Option<Context>,
}

#[derive(Debug)]
struct InnerReceiver {
    #[allow(dead_code)]
    last_tick_was_empty: AtomicBool,
    #[allow(dead_code)]
    known_ssrcs: DashMap<u32, UserId>,
    tick_count: AtomicI64,
    threshold: AtomicI64,
}

impl Receiver {
    pub fn new(guild_id: GuildId, ctx: Context) -> Self {
        Self {
            inner: Arc::new(InnerReceiver {
                last_tick_was_empty: AtomicBool::default(),
                known_ssrcs: DashMap::new(),
                tick_count: Default::default(),
                threshold: AtomicI64::new(2000),
            }),
            guild_id: Some(guild_id),
            ctx: Some(ctx),
        }
    }
}

#[async_trait]
impl VoiceEventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        match ctx {
            EventContext::Track(_) => {}
            EventContext::SpeakingStateUpdate(_) => {
                info!("SpeakingStateUpdate")
            }
            EventContext::VoiceTick(tick) => {
                let speaking = tick.speaking.len();
                let total_participants = speaking + tick.silent.len();
                if total_participants == 0 {
                    return None;
                }
                let tick_count = self.inner.tick_count.load(Ordering::SeqCst);
                let threshold = self.inner.threshold.load(Ordering::SeqCst);
                if tick_count >= threshold {
                    info!("Participants count is {total_participants}");
                    play_random_file(&self.ctx.clone().unwrap(), self.guild_id.unwrap()).await;
                    self.inner.tick_count.store(0, Ordering::SeqCst);
                    let random_number = helpers::get_random_number(0, 2000) as usize;
                    let new_threshold = (random_number * total_participants) as i64;
                    self.inner.threshold.store(new_threshold, Ordering::SeqCst);
                    info!(
                        "Old threshold has been reached. Playing new phrase in {new_threshold}0 ms"
                    );
                }
                self.inner.tick_count.fetch_add(1, Ordering::SeqCst);
            }
            EventContext::RtpPacket(_) => {
                // println!("Rtp packet")
            }
            EventContext::RtcpPacket(_) => {
                // println!("rtcp packet")
            }
            EventContext::ClientDisconnect(_) => {}
            EventContext::DriverConnect(_) => {}
            EventContext::DriverReconnect(_) => {}
            EventContext::DriverDisconnect(_) => {}
            _ => {
                warn!("нихуя")
            }
        };
        None
    }
}

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                error!(
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
    let music_directory =
        env::var("PHRASES_DIRECTORY").expect("Expected a directory in the environment");
    let mut music_files: Vec<_> = read_dir(music_directory)
        .expect("Failed to read music directory")
        .map(|entry| entry.expect("No entries").path())
        .collect();
    let mut rng = thread_rng();
    music_files.shuffle(&mut rng);
    music_files[0].clone()
}

pub async fn join(ctx: &Context, guild_id: GuildId, user_id: &UserId) -> String {
    let guild = guild_id
        .to_guild_cached(&ctx.cache)
        .expect("No cached guild")
        .clone();
    let voice_channel_id = guild
        .voice_states
        .get(user_id)
        .and_then(|voice_state| voice_state.channel_id);
    let voice_channel_id = match voice_channel_id {
        Some(v) => v,
        None => return "Ты должен быть в голосовом канале".to_string(),
    };
    let manager = songbird::get(ctx)
        .await
        .expect("Cannot register songbird manager")
        .clone();
    if let Ok(handler_lock) = manager.join(guild_id, voice_channel_id).await {
        let mut handler = handler_lock.lock().await;
        let evt_receiver = Receiver::new(guild_id, ctx.clone());
        handler.remove_all_global_events();
        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());
        handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver);
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }
    info!("Joined channel");
    "Я тут. Чё надо?".to_string()
}

pub async fn play(options: &[ResolvedOption<'_>], ctx: &Context, guild_id: GuildId) -> String {
    let url = options
        .first()
        .expect("Haven't found any urls")
        .clone()
        .value;
    let url = match url {
        ResolvedValue::String(e) => e.to_string(),
        _ => "Nothing".to_string(),
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
            YoutubeDl::new_search(http_client, url.clone())
        } else {
            YoutubeDl::new(http_client, url.clone())
        };
        handler
            .play_input(src.clone().into())
            .set_volume(0.1)
            .expect("Could not set volume");
        info!("Playing video from {url}");
        format!("Играем {url}")
    } else {
        "Я не в канале".to_string()
    }
}

pub async fn play_file(ctx: &Context, guild_id: GuildId, path: PathBuf) -> String {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let file_path = File::new(path.clone());
        let src = Input::from(file_path);
        handler.enqueue(src.into()).await;
        info!("Added song {path:#?} to queue");
        "Играем".to_string()
    } else {
        "Я не в канале".to_string()
    }
}

pub async fn play_random_file(ctx: &Context, guild_id: GuildId) -> String {
    let path = get_music_file();
    play_file(ctx, guild_id, path).await
}

pub fn register_play() -> CreateCommand {
    CreateCommand::new("play")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "url", "Ссылка на видео")
                .required(true),
        )
        .description("Проиграть с ютуба")
}

pub fn register_join() -> CreateCommand {
    CreateCommand::new("join").description("Присоединиться к чату")
}

pub fn register_phrase() -> CreateCommand {
    CreateCommand::new("phrase").description("Сказать рандомную фразу")
}
