use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Error;
use serenity::all::{Context, Message};
use crate::get_random_number;
use crate::helpers::send_discord_message;

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
            tags
        }
    }

    async fn make_request(self) -> reqwest::Result<Response> {
        let request_url = format!("{}&json={}&limit={}&tags={}", self.url, self.json as i8, self.limit, self.tags.join("+"));
        reqwest::get(request_url).await
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Rule34Model {
    file_url: String,
}

pub async fn find_image(ctx: &Context, msg: &Message) {
    let message_split: Vec<&str> = msg.content.as_str().split(" ").collect();
    if message_split.len() > 1 {
        let search_tag = message_split.get(1).expect("Could not get shit");
        let request_parameters = Rule34Parameters::new(true, 50, vec![search_tag.to_string()]);
        let request = request_parameters.make_request().await.expect("Error trying to call rule34");
        let image = request.text().await.expect("Got no image");
        let model: Result<Vec<Rule34Model>, Error> = serde_json::from_str(image.as_str());
        match model {
            Ok(model) => {
                let random = get_random_number(model.len() as i8).await - 1;
                println!("{model:#?}");
                let url = model.get(random as usize).expect("No model was found").file_url.clone();
                send_discord_message(&ctx, &msg, url.as_str()).await;
            }
            Err(_) => {
                send_discord_message(&ctx, &msg, "Я такую хуйню найти не могу").await;
            }
        }
    } else {
        send_discord_message(&ctx, &msg, "Введи тег для поиска, придурок").await
    }
}