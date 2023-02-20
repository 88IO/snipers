use serenity::{
    builder::CreateApplicationCommand,
    model::{
        mention::Mention,
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            InteractionResponseType
        },
    },
    prelude::*
};
use std::sync::Arc;
use crate::SqliteDatabase;

pub async fn run(ctx: Arc<Context>, command: &ApplicationCommandInteraction) {
    let guild_id = command.guild_id.unwrap();
    let user_id = command.user.id;
    let database = {
        let data_read = ctx.data.read().await;
        data_read.get::<SqliteDatabase>().unwrap().clone()
    };

    database.delete_guild_jobs(user_id, guild_id).await.unwrap();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message|
                    message.content(format!("{}の切断予約を削除しました", Mention::from(user_id)))
                )
        })
        .await
        .unwrap_or_else(|why| println!("cannot respond to slash command: {}", why));
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("clear").description("通話の切断予定を削除します")
}

