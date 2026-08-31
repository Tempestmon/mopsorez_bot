#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Discord(#[from] serenity::Error),
    #[error("{0}")]
    Http(#[from] reqwest::Error),
}
