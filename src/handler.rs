use crate::command_utils::{
    register_commands,
    string_option_ref,
    int_option_ref,
    SnipeMenu
};
use std::str::FromStr;
use crate::job::EventType;
use crate::database::SqliteDatabase;

use std::sync::atomic::{AtomicBool, Ordering};
use serenity::model::guild::{Guild, UnavailableGuild};
use serenity::model::id::{UserId, GuildId};
use serenity::{
    async_trait,
    model::interactions::{
        InteractionResponseType,
        application_command::ApplicationCommandInteraction,
    },
    model::{
        mention::Mention,
        gateway::Ready,
        interactions::Interaction
    },
    prelude::*,
};
use chrono::{Utc, Duration, FixedOffset, DateTime};
use regex::Regex;

const DT_FORMAT: &str = "%m/%d %H:%M:%S (%:z)";

pub struct Handler {
    database: SqliteDatabase,
    re_time: Regex,
    is_loop_running: AtomicBool
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "timezone" => {
                    if let Some(offset) = int_option_ref(&command.data.options, "offset") {
                        self.set_timezone(&ctx, &command, offset).await;
                    } else {
                        self.get_timezone(&ctx, &command).await;
                    }
                },
                "snipe" => {
                    let time = string_option_ref(&command.data.options, "time").unwrap();
                    let kind = string_option_ref(&command.data.options, "kind");
                    self.snipe(&ctx, &command, time, kind).await;
                },
                "display" => {
                    self.display_jobs(&ctx, &command).await;
                },
                "clear" => {
                    let guild_id = command.guild_id.unwrap();
                    let user_id = command.user.id;
                    self.remove_job(&ctx, &command, user_id, guild_id).await;
                },
                _ => println!("not implemented :("),
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) { println!("{} is connected.", ready.user.name);
        // スラッシュコマンド登録
        let commands = register_commands(&ctx).await;

        // ギルド更新
        if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
            for guild in guilds {
                if let Err(why) = self.database.insert_guild_setting(guild.id, 0).await
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

        if self.database.count_jobs().await.unwrap() > 0 {
            if let Ok(_) = self.is_loop_running.compare_exchange(false, true,
                                                                Ordering::Release,
                                                                Ordering::Relaxed)
            {
                self.run_pending(&ctx).await;
            }
        }
    }

    async fn guild_create(&self, _: Context, guild: Guild, is_new: bool) {
        println!("guild_create: {:?}", guild.id);
        if is_new {
            if let Err(why) = self.database.insert_guild_setting(guild.id, 0).await {
                println!("insert guild setting: {:?}", why);
            }
        }
    }

    async fn guild_delete(&self, _: Context, incomplete: UnavailableGuild, _: Option<Guild>) {
        println!("guild_delete {:?}", incomplete.id);
        if !incomplete.unavailable {
            if let Err(why) = self.database.delete_guild_setting(incomplete.id).await {
                println!("insert guild setting: {:?}", why);
            }

        }
    }
}

impl Handler {
    pub async fn new(database: SqliteDatabase, re_time: Regex) -> Self {
        Handler {
            database,
            re_time,
            is_loop_running: AtomicBool::new(false)
        }
    }

    async fn snipe(&self, ctx: &Context, command: &ApplicationCommandInteraction,
                   time: &String, kind: Option<&String>) {
        let guild_id = command.guild_id.unwrap();
        let user_id = command.user.id;

        let target_datetime: DateTime<FixedOffset>;
        if let Some(k) = kind {
            target_datetime = match k.as_str() {
                "oclock" =>  self.absolute_datetime(guild_id, time).await.unwrap(),
                "later" => self.relative_datetime(guild_id, time).await.unwrap(),
                _ => return
            };

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
            let _ = command.defer(&ctx.http).await;

            let msg = command
                .create_followup_message(&ctx.http, |response| {
                    response
                        .content("時間指定方法の選択")
                        .components(|c| c.add_action_row(SnipeMenu::action_row()))
                })
                .await
                .unwrap();

            let mci = match msg.await_component_interaction(ctx)
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
                    } };

            target_datetime = match SnipeMenu::from_str(mci.data.custom_id.as_str()).unwrap() {
                SnipeMenu::Absolute => {
                    self.absolute_datetime(guild_id, time).await.unwrap()
                },
                SnipeMenu::Relative => {
                    self.relative_datetime(guild_id, time).await.unwrap()
                }
            };

            command
                .edit_original_interaction_response(&ctx.http, |message| {
                    message
                        .components(|c| c)
                        .content(format!("{}を{}に切断します",
                                        Mention::from(user_id), target_datetime.format(DT_FORMAT)))
                })
                .await
                .unwrap();
        };

        self.add_job(target_datetime, user_id, guild_id).await;

        if let Ok(_) = self.is_loop_running.compare_exchange(
            false, true,
            Ordering::Release, Ordering::Relaxed)
        {
            println!("start loop");
            self.run_pending(ctx).await;
        }
    }

    async fn set_timezone(&self, ctx: &Context, command: &ApplicationCommandInteraction, offset: &i64) {
        let guild_id = command.guild_id.unwrap();
        let timezone = FixedOffset::east(3600 * *offset as i32);

        if let Ok(_) = self.database.update_guild_setting(guild_id, offset).await {
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

    async fn get_timezone(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        let guild_id = command.guild_id.unwrap();

        if let Ok(setting) = self.database.get_guild_setting(guild_id).await {
            command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message|
                            message.content(format!("{}に設定しました", FixedOffset::east(3600 * setting.utc_offset)))
                        )
                })
                .await
                .unwrap_or_else(|why| println!("cannot respond to slash command: {}", why));
        }
    }

    async fn relative_datetime(&self, guild_id: GuildId, interval: &String) -> Option<DateTime<FixedOffset>> {
        let caps = self.re_time.captures(&interval)?;
        let hour: i64 = caps["hour"].parse().unwrap();
        let minute: i64 = caps["minute"].parse().unwrap();

        let guild_setting = self.database.get_guild_setting(guild_id).await.unwrap();

        let datetime_utc = Utc::now() + Duration::hours(hour) + Duration::minutes(minute);
        Some(datetime_utc.with_timezone(&FixedOffset::east(3600 * guild_setting.utc_offset)))
    }

    async fn absolute_datetime(&self, guild_id: GuildId, time: &String) -> Option<DateTime<FixedOffset>> {
        let caps = self.re_time.captures(time).unwrap();
        let hour: u32 = caps["hour"].parse().unwrap();
        let minute: u32 = caps["minute"].parse().unwrap();

        let guild_setting = self.database.get_guild_setting(guild_id).await.unwrap();

        let tmp_datetime = Utc::today()
            .with_timezone(&FixedOffset::east(3600 * guild_setting.utc_offset))
            .and_hms(hour, minute, 0);

        if Utc::now() >= tmp_datetime {
            Some(tmp_datetime + Duration::days(1))
        } else {
            Some(tmp_datetime)
        }
    }

    async fn add_job(&self, datetime: DateTime<FixedOffset>, user_id: UserId, guild_id: GuildId) {
        let naive_utc = datetime.naive_utc();

        // 切断前通知予約
        let before3min = naive_utc - Duration::minutes(3);
        if before3min < Utc::now().naive_utc() {
            if let Err(why) = self.database
                .insert_job(before3min, user_id,
                            guild_id, EventType::Notification3Min)
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
    }

    async fn display_jobs(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
        let guild_id = command.guild_id.unwrap();

        let jobs = self.database.get_guild_jobs(guild_id).await.unwrap();

        command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message|
                        message.embed(|embed| {
                            jobs.iter().fold(
                                embed
                                    .title("射殺予定")
                                    .description("snipebotの通話切断予定表"),
                                |e, job|
                                    e.field(job.datetime().format(DT_FORMAT), Mention::from(job.userid()), false)
                            )
                        })
                    )
            })
            .await
            .unwrap_or_else(|why| println!("cannot respond to slash command: {}", why));
    }

    async fn remove_job(&self, ctx: &Context, command: &ApplicationCommandInteraction,
                        user_id: UserId, guild_id: GuildId) {
        self.database.delete_guild_jobs(user_id, guild_id).await.unwrap();

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

    async fn run_pending(&self, ctx: &Context) {
        while self.database.count_jobs().await.unwrap() > 0 {
            println!("loop...");
            let jobs = self.database.pull_executables().await.unwrap();

            for job in jobs {
                println!("{:#?}", job);
                match job.event_type {
                    EventType::Disconnect => {
                        if let Ok(_) = job.disconnect(&ctx).await {
                            job.direct_message(&ctx, |m| {
                                m.content(format!("{}に通話を強制切断しました", job.datetime().format(DT_FORMAT)))
                            }).await.unwrap();
                        }
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
