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
        }
    }
}

impl TypeMapKey for Config {
    type Value = Config;
}
