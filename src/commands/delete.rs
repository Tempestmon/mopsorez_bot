use std::num::{NonZeroI64, NonZeroU64};
use serenity::all::{ChannelId, CommandOptionType, Context, CreateCommand, CreateCommandOption, ResolvedOption};

pub async fn delete_messages(options: &[ResolvedOption<'_>], ctx: Context) -> String {
    // let guild = ctx.http.delete_messages(ChannelId(NonZeroU64::try_from(831763438618542113).unwrap()), /* &Value */, /* std::option::Option<&str> */)
    // println!("${guild:#?}");
    String::from("Хуй тебе")
}

pub fn register() -> CreateCommand {
    CreateCommand::new("delete")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "number",
            "Количество сообщений для удаления")
            .required(true)
        )
        .description("Удалить последние сообщения")
}