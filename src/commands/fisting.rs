use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue, User,
};

pub async fn perform_fisting(options: &[ResolvedOption<'_>], user: &User) -> String {
    let current_user = user;
    let fisted_user = options
        .first()
        .expect("No options for fisting")
        .clone()
        .value;
    let fisted_user = match fisted_user {
        ResolvedValue::User(f_u, _) => Ok(f_u),
        _ => Err("Not a user"),
    };
    let fisted_user = fisted_user.unwrap();
    format!("{current_user} профистинговал {fisted_user}")
}

pub fn defend_from_fisting() {
    todo!()
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
