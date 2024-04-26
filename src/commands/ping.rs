use serenity::builder::CreateCommand;

pub fn run() -> String {
    "Ну и хули ты меня пингуешь?".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("Проверить бота на живость")
}
