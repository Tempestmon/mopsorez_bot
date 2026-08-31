use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue, User,
};
use tracing::{debug, info};

use crate::domain::fisting as domain;
use crate::error::BotError;
use crate::infrastructure::fisting_repository::FistingRepository;

pub async fn perform_fisting(
    options: &[ResolvedOption<'_>],
    user: &User,
    repo: &dyn FistingRepository,
) -> Result<String, BotError> {
    let fisted_user = options
        .first()
        .expect("No options for fisting")
        .clone()
        .value;
    let fisted_user = match fisted_user {
        ResolvedValue::User(f_u, _) => f_u,
        _ => panic!("Expected a user option"),
    };
    let records = repo.load()?;
    if domain::is_protected(&fisted_user.name, &records) {
        info!("Protection active for {}", fisted_user.name);
        return Ok(format!(
            "{user} не смог профистинговать {fisted_user}, потому что у него стоит защита"
        ));
    }
    Ok(format!("{user} успешно профистинговал {fisted_user}"))
}

pub async fn defend_from_fisting(
    user: &User,
    repo: &dyn FistingRepository,
) -> Result<String, BotError> {
    let mut records = repo.load()?;
    domain::update_defense(user.name.clone(), &mut records);
    debug!("{records:#?}");
    repo.save(&records)?;
    Ok(format!("{user} защитился от фистинга"))
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
