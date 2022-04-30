use crate::command_utils::{
    register_commands,
    string_option_ref,
    int_option_ref
};
use crate::job::{Job, EventType};
use crate::database::SqliteDatabase;

use std::sync::atomic::{AtomicBool, Ordering};
use serenity::{
    async_trait,
    model::interactions::{
        InteractionResponseType,
        application_command::ApplicationCommandInteraction,
    },
    model::{
        misc::Mention,
        gateway::Ready,
        interactions::Interaction
    },
    prelude::*,
};
use chrono::{Utc, Duration, FixedOffset, TimeZone};
use regex::Regex;


pub struct Handler {
    database: SqliteDatabase,
    re_time: Regex,
    is_loop_running: AtomicBool
}
#[async_trait] impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "snipe" => {
                    println!("{}", command.data.options.len());
                },
                "timezone" => {
                    self.set_timezone(&ctx, &command).await;
                },
                "snipe_at" => {
                    self.snipe_at(&ctx, &command).await;
                },
                "snipe_in" => {
                    self.snipe_in(&ctx, &command).await;
                },
                _ => println!("not implemented :("),
            }

            /*
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("cannot respond to slash command: {}", why);
            }
            */
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) { println!("{} is connected.", ready.user.name);
        // スラッシュコマンド登録
        let commands = register_commands(&ctx).await;

        // ギルド更新
        if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
            for guild in guilds {
                if let Err(why) = self.database.insert_setting(guild.id, 0).await
                {
                    println!("Already exists: {}, {}", guild.id, why);
                }
            }
        }

        // 起動時設定表示
        println!("create global commands: {:#?}", commands);
        if let Ok(settings) = self.database.get_settings().await
        {
            println!("{:#?}", settings);
        }
    }
}

impl Handler {
    pub async fn new(filename: &str) -> Self {
        let database = SqliteDatabase::new(filename).await;
        let re_time = Regex::new(r"(?P<hour>\d{1,2}):(?P<minute>\d{1,2})").unwrap();

        if let Ok(_) = database.pull_executables().await {
            println!("Delete previous jobs.");
        }

        Handler {
            database,
            re_time,
            is_loop_running: AtomicBool::new(false)
        }
    }

    async fn set_timezone(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        let guild_id = command.guild_id.unwrap();
        let offset = int_option_ref(&command.data.options, "offset").unwrap();
        let timezone = FixedOffset::east(3600 * *offset as i32);

        if let Ok(_) = self.database.update_setting(guild_id, offset).await {
            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(format!("{}に設定しました", timezone)))
                })
                .await
            {
                println!("cannot respond to slash command: {}", why);
            }
        }
    }

    async fn snipe_in(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        let guild_id = command.guild_id.unwrap();
        let user_id = command.user.id;

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

        let caps = self.re_time.captures(&interval).unwrap();
        let hour: i64 = caps["hour"].parse().unwrap();
        let minute: i64 = caps["minute"].parse().unwrap();

        let guild_setting = self.database.get_guild_setting(guild_id).await.unwrap();

        let target_datetime = {
            let datetime_utc = Utc::now() + Duration::hours(hour) + Duration::minutes(minute);
            datetime_utc.with_timezone(&FixedOffset::east(3600 * guild_setting.utc_offset))
        };


        let naive_utc = target_datetime.naive_utc();

        // 切断前通知予約
        let before3min = naive_utc - Duration::minutes(3);
        if before3min < Utc::now().naive_utc() {
            if let Err(why) = self.database
                .insert_job(before3min, command.user.id,
                            command.guild_id.unwrap(), EventType::Disconnect)
                .await
            {
                println!("{:?}", why);
            }
        }

        // 切断予約
        if let Err(why) = self.database
            .insert_job(naive_utc, user_id, guild_id, EventType::Disconnect)
            .await
        {
            println!("{:?}", why);
        }

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message|
                        message.content(format!("{}を{}に切断します", Mention::from(command.user.id), target_datetime)))
            })
            .await {
            println!("cannot respond to slash command: {}", why);
        }

        if let Ok(_) = self.is_loop_running.compare_exchange(
            false, true,
            Ordering::Relaxed, Ordering::Relaxed
            )
        {
            self.run_pending(ctx).await;
        }
    }

    async fn snipe_at(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        let guild_id = command.guild_id.unwrap();

        let time = string_option_ref(&command.data.options, "time").unwrap();

        let caps = self.re_time.captures(time).unwrap();
        let hour: u32 = caps["hour"].parse().unwrap();
        let minute: u32 = caps["minute"].parse().unwrap();

        let guild_setting = self.database.get_guild_setting(guild_id).await.unwrap();

        let target_datetime = {
            let tmp_datetime = Utc::today()
                .with_timezone(&FixedOffset::east(3600 * guild_setting.utc_offset))
                .and_hms(hour, minute, 0);

            if Utc::now() >= tmp_datetime {
                tmp_datetime + Duration::days(1)
            } else {
                tmp_datetime
            }
        };

        let naive_utc = target_datetime.naive_utc();

        // 切断前通知予約
        let before3min = naive_utc - Duration::minutes(3);
        if before3min < Utc::now().naive_utc() {
            if let Err(why) = self.database
                .insert_job(before3min, command.user.id,
                            command.guild_id.unwrap(), EventType::Disconnect)
                .await
            {
                println!("{:?}", why);
            }
        }

        // 切断予約
        if let Err(why) = self.database
            .insert_job(naive_utc, command.user.id,
                        command.guild_id.unwrap(), EventType::Disconnect)
            .await
        {
            println!("{:?}", why);
        }

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(format!("{}を{}に切断します", Mention::from(command.user.id), target_datetime)))
            })
            .await
        {
            println!("cannot respond to slash command: {}", why);
        }

        if let Ok(_) = self.is_loop_running.compare_exchange(
            false, true,
            Ordering::Relaxed, Ordering::Relaxed
            )
        {
            self.run_pending(ctx).await;
        }
    }

    async fn run_pending(&self, ctx: &Context) {
        while self.database.count_jobs().await.unwrap() != 0 {
            let jobs = self.database.pull_executables().await.unwrap();

            for job in jobs {
                println!("{:#?}", job);
                match job.event_type {
                    EventType::Disconnect => {
                        let datetime = FixedOffset::east(3600 * job.utc_offset)
                            .from_utc_datetime(&job.naive_utc);

                        let _ = job.direct_message(&ctx, |m| {
                            m.content(format!("{}に通話を強制切断しました", datetime))
                        }).await;

                        let _ = job.disconnect(&ctx).await;
                    },
                    EventType::Notification3Min => {
                        let _ = job.direct_message(&ctx, |m| {
                            m.content("3分後に通話を強制切断します")
                        }).await;
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }

        self.is_loop_running.store(false, Ordering::Relaxed);
    }
}
