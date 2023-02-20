use serenity::{
    builder::CreateApplicationCommand,
    model::{
        id::GuildId,
        application::{
            command::CommandOptionType,
            interaction::{
                application_command::ApplicationCommandInteraction,
                InteractionResponseType
            },
        },
    },
    prelude::*
};
use chrono::FixedOffset;
use std::sync::Arc;
use crate::SqliteDatabase;
use crate::commands::utils::int_option_ref;

async fn set_timezone(database: Arc<SqliteDatabase>, guild_id: GuildId, offset: i64) -> String {
    let timezone = FixedOffset::east_opt(3600 * offset as i32).unwrap();

    if let Ok(_) = database.update_guild_setting(guild_id, offset).await {
        format!("{}に設定しました", timezone)
    } else {
        "タイムゾーンの設定に失敗しました".to_string()
    }
}

async fn get_timezone(database: Arc<SqliteDatabase>, guild_id: GuildId) -> String {
    if let Ok(setting) = database.get_guild_setting(guild_id).await {
        format!("{}に設定されています", FixedOffset::east_opt(3600 * setting.utc_offset).unwrap())
    } else {
        "タイムゾーンの取得に失敗しました".to_string()
    }
}

pub async fn run(ctx: Arc<Context>, command: &ApplicationCommandInteraction) {
    let guild_id = command.guild_id.unwrap();
    let database = {
        let data_read = ctx.data.read().await;
        data_read.get::<SqliteDatabase>().unwrap().clone()
    };

    let content = if let Some(offset) = int_option_ref(&command.data.options, "offset") {
        set_timezone(database, guild_id, *offset).await
    } else {
        get_timezone(database, guild_id).await
    };

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message|
                    message.content(content)
                )
        })
        .await
        .unwrap_or_else(|why| println!("cannot respond to slash command: {}", why));
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("timezone").description("UTCからの時差を設定/表示します")
        .create_option(|option| {
            option
                .name("offset")
                .description("時差（h）")
                .kind(CommandOptionType::Integer)
                .min_int_value(-12)
                .max_int_value(12)
                .required(false)
        })
}

