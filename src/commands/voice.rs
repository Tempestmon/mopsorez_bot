use std::fs::read_dir;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::all::{
    ChannelId, CommandOptionType, CreateCommand, CreateCommandOption, GuildId, ResolvedOption,
    ResolvedValue, UserId,
};
use serenity::async_trait;
use serenity::prelude::Context;
use songbird::events::TrackEvent;
use songbird::input::{File, Input, YoutubeDl};
use songbird::{CoreEvent, Event, EventContext, EventHandler as VoiceEventHandler};
use tracing::{error, info, warn};

use crate::{config::Config, HttpKey};

pub(crate) struct TrackErrorNotifier;

#[derive(Clone, Debug)]
struct Receiver {
    inner: Arc<InnerReceiver>,
    guild_id: Option<GuildId>,
    channel_id: Option<ChannelId>,
    ctx: Option<Context>,
}

#[derive(Debug)]
struct InnerReceiver {
    silence_ticks: AtomicI64,
    cooldown_remaining: AtomicI64,
    // Config values stored in ticks (1 tick ≈ 20ms)
    min_silence_ticks: i64,
    base_cooldown_ticks: i64,
    cooldown_per_person_ticks: i64,
    silence_break_ticks: i64,
}

impl Receiver {
    pub fn new(
        guild_id: GuildId,
        channel_id: ChannelId,
        ctx: Context,
        min_silence_ticks: i64,
        base_cooldown_ticks: i64,
        cooldown_per_person_ticks: i64,
        silence_break_ticks: i64,
    ) -> Self {
        Self {
            inner: Arc::new(InnerReceiver {
                silence_ticks: AtomicI64::new(0),
                cooldown_remaining: AtomicI64::new(0),
                min_silence_ticks,
                base_cooldown_ticks,
                cooldown_per_person_ticks,
                silence_break_ticks,
            }),
            guild_id: Some(guild_id),
            channel_id: Some(channel_id),
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
                let total = speaking + tick.silent.len();
                if total == 0 {
                    return None;
                }

                let cooldown = self.inner.cooldown_remaining.load(Ordering::SeqCst);
                if cooldown > 0 {
                    self.inner.cooldown_remaining.store(cooldown - 1, Ordering::SeqCst);
                }

                if speaking > 0 {
                    let silence = self.inner.silence_ticks.load(Ordering::SeqCst);
                    self.inner.silence_ticks.store(0, Ordering::SeqCst);

                    // React when someone speaks after ≥1 second of silence.
                    // Short pauses between speakers (< 50 ticks = 1s) are ignored,
                    // so in multi-person conversations the bot doesn't fire on every breath.
                    if silence > self.inner.min_silence_ticks && cooldown == 0 {
                        info!("Speech detected after {silence} ticks of silence, reacting");
                        if let (Some(ctx), Some(guild_id)) = (&self.ctx, self.guild_id) {
                            play_random_file(ctx, guild_id).await;
                        }
                        let new_cooldown = self.inner.base_cooldown_ticks
                            + (total as i64 - 1) * self.inner.cooldown_per_person_ticks;
                        self.inner.cooldown_remaining.store(new_cooldown, Ordering::SeqCst);
                    }
                } else {
                    let silence = self.inner.silence_ticks.fetch_add(1, Ordering::SeqCst) + 1;

                    if silence >= self.inner.silence_break_ticks && cooldown == 0 {
                        info!("Breaking {silence}-tick silence");
                        if let (Some(ctx), Some(guild_id)) = (&self.ctx, self.guild_id) {
                            play_random_file(ctx, guild_id).await;
                        }
                        self.inner.silence_ticks.store(0, Ordering::SeqCst);
                        self.inner.cooldown_remaining.store(self.inner.base_cooldown_ticks, Ordering::SeqCst);
                    }
                }
            }
            EventContext::RtpPacket(_) => {}
            EventContext::RtcpPacket(_) => {}
            EventContext::ClientDisconnect(_) => {}
            EventContext::DriverConnect(_) => {}
            EventContext::DriverReconnect(_) => {}
            EventContext::DriverDisconnect(_) => {
                if let (Some(ctx), Some(guild_id), Some(channel_id)) =
                    (&self.ctx, self.guild_id, self.channel_id)
                {
                    let ctx = ctx.clone();
                    let receiver = self.clone();
                    tokio::spawn(async move {
                        // Wait for any concurrent join() to complete before deciding to rejoin
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        let manager = match songbird::get(&ctx).await {
                            Some(m) => m,
                            None => {
                                error!("Songbird not in context, cannot auto-rejoin");
                                return;
                            }
                        };
                        if manager.get(guild_id).is_some() {
                            return; // Already reconnected by someone else
                        }
                        warn!("Auto-rejoining channel {channel_id} after driver disconnect");
                        match manager.join(guild_id, channel_id).await {
                            Ok(handler_lock) => {
                                let mut handler = handler_lock.lock().await;
                                handler.remove_all_global_events();
                                handler.add_global_event(
                                    CoreEvent::SpeakingStateUpdate.into(),
                                    receiver.clone(),
                                );
                                handler.add_global_event(
                                    CoreEvent::RtpPacket.into(),
                                    receiver.clone(),
                                );
                                handler.add_global_event(
                                    CoreEvent::RtcpPacket.into(),
                                    receiver.clone(),
                                );
                                handler.add_global_event(
                                    CoreEvent::VoiceTick.into(),
                                    receiver.clone(),
                                );
                                handler.add_global_event(
                                    CoreEvent::DriverDisconnect.into(),
                                    receiver,
                                );
                                handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
                                info!("Auto-rejoin complete, events re-registered");
                            }
                            Err(e) => error!("Auto-rejoin failed: {e}"),
                        }
                    });
                }
            }
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

fn get_music_file(phrases_directory: &PathBuf) -> Option<PathBuf> {
    let mut music_files: Vec<_> = read_dir(phrases_directory)
        .ok()?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();
    if music_files.is_empty() {
        return None;
    }
    music_files.shuffle(&mut thread_rng());
    Some(music_files[0].clone())
}

pub async fn join(ctx: &Context, guild_id: GuildId, user_id: &UserId) -> String {
    // Ref from cache must be dropped before any .await — keep it in a block
    let voice_channel_id = {
        match guild_id.to_guild_cached(&ctx.cache) {
            Some(guild) => guild.voice_states.get(user_id).and_then(|vs| vs.channel_id),
            None => {
                error!("Guild {guild_id} not found in cache");
                return "Гильдия не найдена".to_string();
            }
        }
    };
    let Some(voice_channel_id) = voice_channel_id else {
        return "Ты должен быть в голосовом канале".to_string();
    };
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not registered")
        .clone();
    // Remove any stale/failed call before joining to avoid state corruption
    let _ = manager.remove(guild_id).await;
    match manager.join(guild_id, voice_channel_id).await {
        Ok(handler_lock) => {
            let mut handler = handler_lock.lock().await;
            const TICKS_PER_SEC: i64 = 50; // 1 tick ≈ 20ms
            let (min_silence, base_cd, cd_per_person, silence_break) = {
                let data = ctx.data.read().await;
                let cfg = data.get::<Config>().expect("Config must be in TypeMap");
                (
                    cfg.voice_min_silence_secs as i64 * TICKS_PER_SEC,
                    cfg.voice_cooldown_secs as i64 * TICKS_PER_SEC,
                    cfg.voice_cooldown_per_person_secs as i64 * TICKS_PER_SEC,
                    cfg.voice_silence_break_secs as i64 * TICKS_PER_SEC,
                )
            };
            let evt_receiver = Receiver::new(guild_id, voice_channel_id, ctx.clone(), min_silence, base_cd, cd_per_person, silence_break);
            handler.remove_all_global_events();
            handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::RtpPacket.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::RtcpPacket.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::VoiceTick.into(), evt_receiver.clone());
            handler.add_global_event(CoreEvent::DriverDisconnect.into(), evt_receiver);
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        }
        Err(e) => {
            error!("Failed to join voice channel: {e}");
            return "Не удалось присоединиться к каналу".to_string();
        }
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
        .expect("Songbird not registered")
        .clone();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("HttpClient not in TypeMap")
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let src = if search {
            YoutubeDl::new_search(http_client, url.clone())
        } else {
            YoutubeDl::new(http_client, url.clone())
        };
        if let Err(e) = handler.play_input(src.into()).set_volume(0.1) {
            error!("Could not set volume: {e}");
        }
        info!("Playing video from {url}");
        format!("Играем {url}")
    } else {
        "Я не в канале".to_string()
    }
}

pub async fn play_file(ctx: &Context, guild_id: GuildId, path: PathBuf) -> String {
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not registered")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let src = Input::from(File::new(path.clone()));
        handler.enqueue(src.into()).await;
        info!("Added song {path:#?} to queue");
        "Играем".to_string()
    } else {
        "Я не в канале".to_string()
    }
}

pub async fn play_random_file(ctx: &Context, guild_id: GuildId) -> String {
    let phrases_directory = {
        let data = ctx.data.read().await;
        data.get::<Config>()
            .expect("Config must be in TypeMap")
            .phrases_directory
            .clone()
    };
    match get_music_file(&phrases_directory) {
        Some(path) => play_file(ctx, guild_id, path).await,
        None => {
            error!("No music files found in {:?}", phrases_directory);
            "Нет файлов для воспроизведения".to_string()
        }
    }
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
