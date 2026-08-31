use serenity::all::{
    ChannelId, CommandOptionType, Context, CreateCommand, CreateCommandOption, GetMessages,
    GuildId, ResolvedOption, ResolvedValue,
};

use crate::error::BotError;

pub async fn delete_messages(
    options: &[ResolvedOption<'_>],
    ctx: &Context,
    guild_id: GuildId,
    channel_id: &ChannelId,
) -> Result<String, BotError> {
    let number = options
        .first()
        .expect("No number was provided")
        .clone()
        .value;
    let number = match number {
        ResolvedValue::Integer(e) => e,
        _ => 0,
    };
    if number >= 10 {
        return Ok(String::from("Слишком дохуя ты решил удалить"));
    }
    if number <= 0 {
        return Ok(String::from("Нельзя удалить 0 сообщений, дебил"));
    }
    // GuildRef from to_guild_cached is !Send — drop it before any .await
    let channel_opt = {
        let Some(guild) = guild_id.to_guild_cached(&ctx.cache) else {
            return Ok("Гильдия не найдена".to_string());
        };
        guild.channels.get(channel_id).cloned()
    };
    let Some(channel) = channel_opt else {
        return Ok("Канал не найден".to_string());
    };
    let builder = GetMessages::new().limit(number as u8);
    let messages = channel.messages(&ctx.http, builder).await?;
    channel.delete_messages(&ctx.http, messages.into_iter()).await?;
    Ok(format!("Сообщения удалены, их количество: {number}"))
}

pub fn register() -> CreateCommand {
    CreateCommand::new("delete")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "number",
                "Количество сообщений для удаления",
            )
            .required(true),
        )
        .description("Удалить последние сообщения")
}
