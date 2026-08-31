use rand::seq::SliceRandom;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use tracing::info;

use crate::error::BotError;

struct Rule34Parameters {
    url: String,
    json: bool,
    limit: i8,
    tags: Vec<String>,
}

impl Rule34Parameters {
    fn new(json: bool, limit: i8, tags: Vec<String>) -> Self {
        Rule34Parameters {
            url: String::from("https://api.rule34.xxx/index.php?page=dapi&s=post&q=index"),
            json,
            limit,
            tags,
        }
    }

    async fn make_request(self) -> reqwest::Result<Response> {
        let request_url = format!(
            "{}&json={}&limit={}&tags={}",
            self.url,
            self.json as i8,
            self.limit,
            self.tags.join("+")
        );
        reqwest::get(request_url).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Rule34Model {
    file_url: String,
}

pub async fn find_image(options: &[ResolvedOption<'_>]) -> Result<String, BotError> {
    let tag = options.first().expect("No tag was provided").clone().value;
    let tag = match tag {
        ResolvedValue::String(s) => s,
        _ => panic!("Expected a string option"),
    };
    info!("tag is {tag:?}");
    let request_parameters =
        Rule34Parameters::new(true, 50, vec![tag.to_string()]);
    let response = request_parameters.make_request().await?;
    let body = response.text().await?;
    if body.is_empty() {
        return Ok(String::from("Такой хуйни я найти не могу"));
    }
    let images: Vec<Rule34Model> = serde_json::from_str(&body)?;
    let url = images
        .choose(&mut rand::thread_rng())
        .map(|m| m.file_url.clone())
        .unwrap_or_else(|| "Такой хуйни я найти не могу".to_string());
    Ok(url)
}

pub fn register() -> CreateCommand {
    CreateCommand::new("rule34")
        .description("Поискать красивые картинки")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "tag", "Тег для поиска")
                .required(true),
        )
        .nsfw(true)
}
