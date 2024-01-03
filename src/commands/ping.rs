use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

pub fn run(_: &[ResolvedOption]) -> String {
    "Ну и хули ты меня пингуешь?".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping")
        .description("Проверить бота на живость")
}
