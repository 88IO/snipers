use serenity::{
    model::interactions::{
        InteractionResponseType,
        application_command::ApplicationCommandInteraction,
    },
    prelude::Context
};
use chrono::{Utc, Duration};
use crate::commands::job::EventType;
use crate::Handler;

pub async fn snipe_in(handler: &Handler, ctx: &Context, command: &ApplicationCommandInteraction) {
    let guild_id = command.guild_id.unwrap().0 as i64;
    let user_id = command.user.id.0 as i64;

    let interval = command
        .data
        .options
        .iter()
        .find(|&v| v.name == "interval")
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .to_string();

    let caps = handler.re_time.captures(&interval).unwrap();
    let hour: i64 = caps["hour"].parse().unwrap();
    let minute: i64 = caps["minute"].parse().unwrap();

    let datetime = Utc::now() + Duration::hours(hour) + Duration::minutes(minute);
    let naive_utc = datetime.naive_utc();

    if let Err(why) = sqlx::query!(
        "INSERT INTO job (naive_utc, user_id, guild_id, event_type, utc_offset) SELECT ?, ?, ?, ?, utc_offset FROM setting WHERE guild_id=?",
        naive_utc, user_id, guild_id, EventType::Disconnect, guild_id
        )
        .execute(&handler.database)
        .await
    {
        println!("{:?}", why);
    }

    if let Err(why) = command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Success"))
        })
        .await
    {
        println!("cannot respond to slash command: {}", why);
    }
}

