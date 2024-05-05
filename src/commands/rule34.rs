use rand::seq::SliceRandom;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};

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

pub async fn find_image(options: &[ResolvedOption<'_>]) -> String {
    let tag = options.first().expect("No tag was provided").clone().value;
    let tag = match tag {
        ResolvedValue::String(s) => Ok(s),
        _ => Err("Not a string"),
    };
    println!("tag is {tag:?}");
    let request_parameters =
        Rule34Parameters::new(true, 50, vec![tag.expect("No tags found").to_string()]);
    let response = request_parameters
        .make_request()
        .await
        .expect("Got error receiving response");
    let response = response.text().await.expect("Got error converting to text");
    if response.is_empty() {
        return String::from("Такой хуйни я найти не могу");
    }
    let images: Result<Vec<Rule34Model>, Error> = serde_json::from_str(&*response);
    let images = images.expect("Got error parsing response");
    String::from(
        images
            .choose(&mut rand::thread_rng())
            .expect("Couldn't choose random")
            .file_url
            .clone(),
    )
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
