use serenity::{
    builder::CreateApplicationCommand,
    model::{
        mention::Mention,
        id::{GuildId, UserId},
        application::{
            command::CommandOptionType,
            interaction::application_command::ApplicationCommandInteraction,
        }
    },
    prelude::*
};
use std::{sync::{Arc, atomic::Ordering}, collections::HashSet};
use chrono::{Utc, Duration, FixedOffset, DateTime, Timelike};
use regex::{Regex, Match};
use tokio::sync::OnceCell;
use crate::job::EventType;
use crate::{SqliteDatabase, JobRunner};
use crate::commands::utils::*;

static RE_TIME: OnceCell<Regex> = OnceCell::const_new();

pub async fn run(ctx: Arc<Context>, command: &ApplicationCommandInteraction) {
    let re_time = RE_TIME.get_or_init(|| async {
        Regex::new(r"(?:(?P<hour>\d{1,2})(?:時間|時|:|：|hours|hour|h|Hours|Hour|H|\s^@))?(?:(?P<minute>\d{1,2})(?:分|mins|min|m|Mins|Min|M|))?").unwrap()}).await;
    let time = string_option_ref(&command.data.options, "time").unwrap();
    let caps = re_time.captures(time).unwrap();
    let hour = caps.name("hour");
    let minute = caps.name("minute");

    if hour.is_none() && minute.is_none() {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .interaction_response_data(|message| {
                        message
                            .components(|c| c)
                            .content("時間/時刻を認識できません")
                    })
            })
            .await
            .unwrap();

        return;
    }

    let types = string_option_ref(&command.data.options, "type");
    let opt_user = user_option_ref(&command.data.options, "user");
    let opt_role = role_option_ref(&command.data.options, "role");

    let guild_id = command.guild_id.unwrap();
    let user_id = command.user.id;

    let mut target_userids: HashSet<UserId> = HashSet::new();
    if let Some(u) = opt_user {
        target_userids.insert(u.id);
    }
    if let Some(r) = opt_role {
        let guild_members = r.guild_id.members(&ctx.http, None, None).await.unwrap();
        guild_members.iter().for_each(|member| {
            if member.roles.contains(&r.id) {
                target_userids.insert(member.user.id);
            }
        });
    }
    if target_userids.is_empty() {
        target_userids.insert(user_id);
    }

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

    let target_users_str: String = target_userids.iter().map(|&ui| Mention::from(ui).to_string()).collect();
    for u in target_userids.into_iter() {
        add_job(database.clone(), target_datetime, u, guild_id).await;
    }

    if types.is_some() {
        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .interaction_response_data(|message| {
                        message
                            .components(|c| c)
                            .content(format!("{0}を<t:{1}:T> (<t:{1}:R>)に切断します",
                                            target_users_str, target_datetime.timestamp()))
                    })
            })
            .await
            .unwrap();
    } else {
        command
            .edit_original_interaction_response(&ctx.http, |message| {
                message
                    .components(|c| c)
                    .content(format!("{0}を<t:{1}:T> (<t:{1}:R>)に切断します",
                                    target_users_str, target_datetime.timestamp()))
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

fn relative_datetime(h_opt: Option<Match>, m_opt: Option<Match>, utc_offset: i32) -> DateTime<FixedOffset> {
    let hour: u32 = if let Some(h) = h_opt { h.as_str().parse().unwrap() } else { 0 };
    let minute: u32 = if let Some(m) = m_opt { m.as_str().parse().unwrap() } else { 0 };
    let datetime_utc = Utc::now() + Duration::hours(hour.into()) + Duration::minutes(minute.into());
    datetime_utc.with_timezone(&FixedOffset::east_opt(3600 * utc_offset).unwrap())
}

fn absolute_datetime(h_opt: Option<Match>, m_opt: Option<Match>, utc_offset: i32) -> DateTime<FixedOffset> {
    let utc_now = Utc::now();
    let hour: u32 = if let Some(h) = h_opt { h.as_str().parse().unwrap() } else { utc_now.hour() };
    let minute: u32 = if let Some(m) = m_opt { m.as_str().parse().unwrap() } else { utc_now.minute() };
    let mut tmp_datetime = DateTime::<FixedOffset>::from_utc(
        utc_now.date_naive().and_hms_opt(hour, minute, 0).unwrap(),
        FixedOffset::east_opt(3600 * utc_offset).unwrap());

    if h_opt.is_none() && utc_now.minute() > tmp_datetime.minute() {
        tmp_datetime += Duration::hours(1);
    } else if utc_now >= tmp_datetime {
        tmp_datetime += Duration::days(1);
    }

    tmp_datetime
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
                .description("切断する時刻/切断するまでの時間")
                .kind(CommandOptionType::String)
                .required(true)
        })
        .create_option(|option| {
            option
                .name("type")
                .description("指定方法を選択します (at: 時刻, in: 時間後)")
                .kind(CommandOptionType::String)
                .add_string_choice("at", "at")
                .add_string_choice("in", "in")
                .required(false)
        })
        .create_option(|option| {
            option
                .name("role")
                .description("Roleに対して切断予約します")
                .kind(CommandOptionType::Role)
                .required(false)
        })
        .create_option(|option| {
            option
                .name("user")
                .description("ユーザーに対して切断予約します")
                .kind(CommandOptionType::User)
                .required(false)
        })
}
