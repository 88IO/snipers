use serenity::{
    builder::CreateApplicationCommand,
    model::{
        prelude::Mention,
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            InteractionResponseType
        },
    },
    prelude::*
};
use std::sync::Arc;
use crate::job::EventType;
use crate::commands::utils::*;
use crate::SqliteDatabase;

pub async fn run(ctx: Arc<Context>, command: &ApplicationCommandInteraction) {
    let guild_id = command.guild_id.unwrap();
    let database = {
        let data_read = ctx.data.read().await;
        data_read.get::<SqliteDatabase>().unwrap().clone()
    };

    let jobs = database.get_guild_jobs(guild_id).await.unwrap();

    command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message|
                    message.embed(|embed| {
                        jobs.iter()
                            .filter(|job| job.event_type == EventType::Disconnect)
                            .fold(embed
                                    .title("射殺予定")
                                    .description("snipebotの通話切断予定表"),
                                    |e, job|
                                    e.field(job.datetime().format(DT_FORMAT),
                                            Mention::from(job.userid()),
                                            false)
                        )
                    })
                )
        })
        .await
        .unwrap_or_else(|why| println!("cannot respond to slash command: {}", why));
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("show").description("通話の切断予定を表示します")
}

