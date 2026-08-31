use std::path::PathBuf;
use std::sync::Arc;

use serenity::prelude::TypeMapKey;

use crate::domain::fisting::FistingInfo;
use crate::error::BotError;

pub trait FistingRepository: Send + Sync {
    fn load(&self) -> Result<Vec<FistingInfo>, BotError>;
    fn save(&self, data: &[FistingInfo]) -> Result<(), BotError>;
}

pub struct JsonFistingRepository {
    path: PathBuf,
}

impl JsonFistingRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl FistingRepository for JsonFistingRepository {
    fn load(&self) -> Result<Vec<FistingInfo>, BotError> {
        let file = std::fs::File::open(&self.path)?;
        Ok(serde_json::from_reader(file)?)
    }

    fn save(&self, data: &[FistingInfo]) -> Result<(), BotError> {
        let mut file = std::fs::File::create(&self.path)?;
        serde_json::to_writer_pretty(&mut file, data)?;
        Ok(())
    }
}

pub struct FistingRepoKey;

impl TypeMapKey for FistingRepoKey {
    type Value = Arc<dyn FistingRepository>;
}
