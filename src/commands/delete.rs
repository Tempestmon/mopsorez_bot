use std::num::{NonZeroI64, NonZeroU64};
use std::ops::Deref;
use std::sync::Arc;
use serenity::all::{ChannelId, CommandOptionType, Context, CreateCommand, CreateCommandOption, GetMessages, GuildId, ResolvedOption, ResolvedValue, User};

pub async fn delete_messages(options: &[ResolvedOption<'_>], ctx: &Context, guild_id: &Arc<GuildId>, user: &User, channel_id: &ChannelId) -> String {
    let number = options.first().expect("No number was provided").clone().value;
    let number = match number {
        ResolvedValue::Integer(e) => {e}
        _ => {0}
    };
    let guild = guild_id
        .to_guild_cached(&ctx.cache)
        .unwrap()
        .clone();
    let channel = guild.channels.get(channel_id).unwrap();
    let builder = GetMessages::new().limit(number as u8);
    let messages = channel.messages(&ctx.http, builder).await.unwrap();
    let delete_result = channel.delete_messages(&ctx.http, messages.into_iter()).await.unwrap();
    // TODO: Обработка всех unwrap
    format!("Я удалил последние {number} сообщений")
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