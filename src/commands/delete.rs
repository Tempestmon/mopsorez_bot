use serenity::all::{ChannelId, CommandOptionType, Context, CreateCommand, CreateCommandOption, GetMessages, GuildId, ResolvedOption, ResolvedValue};

pub async fn delete_messages(options: &[ResolvedOption<'_>], ctx: &Context, guild_id: GuildId, channel_id: &ChannelId) -> String {
    let number = options.first().expect("No number was provided").clone().value;
    let number = match number {
        ResolvedValue::Integer(e) => {e}
        _ => {0}
    };
    if number >= 10 {
        return String::from("Слишком дохуя ты решил удалить")
    }
    if number <= 0 {
        return String::from("Нельзя удалить 0 сообщений, дебил")
    }
    let guild = guild_id
        .to_guild_cached(&ctx.cache)
        .expect("Guild cannot be found")
        .clone();
    let channel = guild
        .channels
        .get(channel_id)
        .expect("Channel is not found");
    let builder = GetMessages::new().limit(number as u8);
    let messages = channel
        .messages(&ctx.http, builder)
        .await
        .expect("No messages found");
    channel.delete_messages(&ctx.http, messages.into_iter())
        .await
        .expect("Got an error deleting messages");
    format!("Сообщения удалены, их количество: {number}")
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