use std::env;
use std::path::PathBuf;

use serenity::prelude::TypeMapKey;

pub struct Config {
    pub discord_token: String,
    pub bot_owner: String,
    pub phrases_directory: PathBuf,
    pub hooli: PathBuf,
    pub pnh: PathBuf,
    pub otvet: PathBuf,
    pub fisting_data: PathBuf,
    /// Minimum silence (seconds) before the bot reacts to someone starting to speak.
    pub voice_min_silence_secs: u64,
    /// Base cooldown (seconds) between voice reactions.
    pub voice_cooldown_secs: u64,
    /// Extra cooldown (seconds) added per participant above 1.
    pub voice_cooldown_per_person_secs: u64,
    /// Silence duration (seconds) after which the bot breaks the silence itself.
    pub voice_silence_break_secs: u64,
}

fn parse_env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            discord_token: env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set"),
            bot_owner: env::var("BOT_OWNER").unwrap_or_else(|_| "tempestmon".to_string()),
            phrases_directory: PathBuf::from(
                env::var("PHRASES_DIRECTORY").expect("PHRASES_DIRECTORY must be set"),
            ),
            hooli: PathBuf::from(env::var("HOOLI").expect("HOOLI must be set")),
            pnh: PathBuf::from(env::var("PNH").expect("PNH must be set")),
            otvet: PathBuf::from(env::var("OTVET").expect("OTVET must be set")),
            fisting_data: PathBuf::from(
                env::var("FISTING_DATA_PATH")
                    .unwrap_or_else(|_| "fisting_info.json".to_string()),
            ),
            voice_min_silence_secs: parse_env_u64("VOICE_MIN_SILENCE_SECS", 1),
            voice_cooldown_secs: parse_env_u64("VOICE_COOLDOWN_SECS", 10),
            voice_cooldown_per_person_secs: parse_env_u64("VOICE_COOLDOWN_PER_PERSON_SECS", 5),
            voice_silence_break_secs: parse_env_u64("VOICE_SILENCE_BREAK_SECS", 30),
        }
    }
}

impl TypeMapKey for Config {
    type Value = Config;
}
