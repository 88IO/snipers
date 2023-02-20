use serenity::{
    builder::CreateApplicationCommand,
    model::{
        mention::Mention,
        id::{GuildId, UserId},
        application::{
            command::CommandOptionType,
            interaction::application_command::ApplicationCommandInteraction,
        },
    },
    prelude::*
};
use std::sync::{Arc, atomic::Ordering};
use chrono::{Utc, Duration, FixedOffset, DateTime};
use regex::Regex;
use tokio::sync::OnceCell;
use crate::job::EventType;
use crate::{SqliteDatabase, JobRunner};
use crate::commands::utils::*;

static RE_TIME: OnceCell<Regex> = OnceCell::const_new();

pub async fn run(ctx: Arc<Context>, command: &ApplicationCommandInteraction) {
    let re_time = RE_TIME.get_or_init(|| async {
        Regex::new(r"(?P<hour>\d{1,2}):(?P<minute>\d{1,2})").unwrap()}).await;
    let time = string_option_ref(&command.data.options, "time").unwrap();
    let caps = re_time.captures(time).unwrap();
    let hour: u32 = caps["hour"].parse().unwrap();
    let minute: u32 = caps["minute"].parse().unwrap();

    let types = string_option_ref(&command.data.options, "type");

    let guild_id = command.guild_id.unwrap();
    let user_id = command.user.id;

    let database = {
        let data_read = ctx.data.read().await;
        data_read.get::<SqliteDatabase>().unwrap().clone()
    };
    let is_loop_running = {
        let data_read = ctx.data.read().await;
        data_read.get::<JobRunner>().unwrap().clone()
    };

    let utc_offset = {
        let guild_setting = database.get_guild_setting(guild_id).await.unwrap();
        guild_setting.utc_offset
    };

    let snipe_type = match types {
        Some(t) => t.to_owned(),
        None => {
            let _ = command.defer(&ctx.http).await;

            let msg = command
                .create_followup_message(&ctx.http, |response| {
                    response
                        .content("時間指定方法の選択")
                        .components(|c| c.add_action_row(SnipeType::action_row()))
                })
                .await
                .unwrap();

            let mci = match msg.await_component_interaction(&*ctx)
                .author_id(user_id)
                .timeout(std::time::Duration::from_secs(30))
                .await {
                    Some(ci) => {
                        ci
                    },
                    None => {
                        command
                            .edit_original_interaction_response(&ctx.http, |message| {
                                message
                                    .components(|c| c)
                                    .content("タイムアウトしました")
                            })
                            .await
                            .unwrap();
                        return;
                    }
                };

            mci.data.custom_id.to_owned()
        }
    };

    let target_datetime: DateTime<FixedOffset> = match snipe_type.as_str() {
        "at" => absolute_datetime(hour, minute, utc_offset),
        "in" => relative_datetime(hour, minute, utc_offset),
        _ => panic!("unexpected SnipeType.")
    };

    add_job(database.clone(), target_datetime, user_id, guild_id).await;

    if types.is_some() {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .interaction_response_data(|message| {
                        message
                            .components(|c| c)
                            .content(format!("{}を{}に切断します",
                                            Mention::from(user_id), target_datetime.format(DT_FORMAT)))
                    })
            })
            .await
            .unwrap();
    } else {
        command
            .edit_original_interaction_response(&ctx.http, |message| {
                message
                    .components(|c| c)
                    .content(format!("{}を{}に切断します",
                                    Mention::from(user_id), target_datetime.format(DT_FORMAT)))
            })
            .await
            .unwrap();
    }


    if let Ok(_) = is_loop_running.compare_exchange(
        false, true,
        Ordering::Release, Ordering::Relaxed)
    {
        println!("start loop");
        JobRunner::run(ctx).await;
    }
}

fn relative_datetime(hour: u32, minute: u32, utc_offset: i32) -> DateTime<FixedOffset> {
    let datetime_utc = Utc::now() + Duration::hours(hour.into()) + Duration::minutes(minute.into());
    datetime_utc.with_timezone(&FixedOffset::east_opt(3600 * utc_offset).unwrap())
}

fn absolute_datetime(hour: u32, minute: u32, utc_offset: i32) -> DateTime<FixedOffset> {
    let tmp_datetime = DateTime::<FixedOffset>::from_local(
        Utc::now().date_naive().and_hms_opt(hour, minute, 0).unwrap(),
        FixedOffset::east_opt(3600 * utc_offset).unwrap());

    if Utc::now() >= tmp_datetime {
        tmp_datetime + Duration::days(1)
    } else {
        tmp_datetime
    }
}

async fn add_job(database: Arc<SqliteDatabase>, datetime: DateTime<FixedOffset>, user_id: UserId, guild_id: GuildId) {
    let naive_utc = datetime.naive_utc();

    // 切断前通知予約
    let before3min = naive_utc - Duration::minutes(3);
    if before3min.timestamp() > Utc::now().timestamp() {
        if let Err(why) = database
            .insert_job(before3min, user_id,
                        guild_id, EventType::Notification3Min)
            .await
        {
            println!("{:?}", why);
        }
    }

    // 切断予約
    if let Err(why) = database
        .insert_job(naive_utc, user_id, guild_id, EventType::Disconnect)
        .await
    {
        println!("{:?}", why);
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("snipe").description("通話の切断予定を削除します")
        .create_option(|option| {
            option
                .name("time")
                .description("切断する絶対/相対時刻")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("type")
                .description("指定方法を選択します")
                .kind(CommandOptionType::String)
                .add_string_choice("at", "at")
                .add_string_choice("in", "in")
        })
}
