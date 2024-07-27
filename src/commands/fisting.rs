use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue, User,
};

#[derive(Deserialize, Serialize, Debug)]
struct FistingInfo {
    user: String,
    fisting_defense_data: DateTime<Utc>,
}

impl FistingInfo {
    fn new(user: String) -> Self {
        FistingInfo {
            user,
            fisting_defense_data: Utc::now(),
        }
    }
}

pub async fn perform_fisting(options: &[ResolvedOption<'_>], user: &User) -> String {
    let fisted_user = options
        .first()
        .expect("No options for fisting")
        .clone()
        .value;
    let fisted_user = match fisted_user {
        ResolvedValue::User(f_u, _) => Ok(f_u),
        _ => Err("Not a user"),
    };
    let fisted_user = fisted_user.expect("Fisted user is not a user");
    let file = std::fs::File::open("fisting_info.json").expect("Couldn't open file");
    let mut fisting_info: Vec<FistingInfo> =
        serde_json::from_reader(file).expect("Couldn't parse fisting info");
    for info in &mut fisting_info {
        if info.user == fisted_user.name {
            let delta = Utc::now().time() - info.fisting_defense_data.time();
            let delta = delta.num_minutes();
            if delta <= 30 {
                return format!(
                    "{user} не смог профистинговать {fisted_user}, потому что у него стоит защита"
                );
            }
        }
    }
    format!("{user} успешно профистинговал {fisted_user}")
}

fn is_username_in_fisting_info(username: &String, fisting_info: &mut Vec<FistingInfo>) -> bool {
    for info in fisting_info {
        if &info.user == username {
            return true;
        }
    }
    false
}

pub async fn defend_from_fisting(user: &User) -> String {
    let username = user.name.clone();
    let file = std::fs::File::open("fisting_info.json").expect("Couldn't open file");
    let mut fisting_info: Vec<FistingInfo> =
        serde_json::from_reader(file).expect("Couldn't parse fisting info");
    if !is_username_in_fisting_info(&username, &mut fisting_info) {
        fisting_info.push(FistingInfo::new(username));
    } else {
        for info in &mut fisting_info {
            if info.user == username {
                info.fisting_defense_data = Utc::now();
            }
        }
    }
    println!("{fisting_info:#?}");
    let mut file = std::fs::File::create("fisting_info.json").unwrap();
    serde_json::to_writer_pretty(&mut file, &fisting_info).unwrap();
    format!("{user} защитился от фистинга")
}

pub fn register_fisting() -> CreateCommand {
    CreateCommand::new("fisting")
        .description("Сделать фистинг")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "Пользователь, которому нужно сделать фистинг",
            )
            .required(true),
        )
}

pub fn register_defense() -> CreateCommand {
    CreateCommand::new("fisting_defense").description("Временно защититься от фистинга")
}
