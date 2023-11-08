mod job;
mod database;
mod commands;
use database::SqliteDatabase;
use job::EventType;

use dotenv::dotenv;
use serenity::{
    async_trait,
    client::Client,
    model::{
        gateway::Ready,
        application::interaction::Interaction,
        application::command::Command,
        guild::{Guild, UnavailableGuild}
    },
    prelude::*,
};
use std::{
    env,
    sync::{Arc, atomic::{AtomicBool, Ordering}}
};

pub struct JobRunner;

impl TypeMapKey for JobRunner {
    type Value = Arc<AtomicBool>;
}

impl JobRunner {
    pub async fn run(ctx: Arc<Context>) {
        let database = {
            let data_read = ctx.data.read().await;
            data_read.get::<SqliteDatabase>().unwrap().clone()
        };
        let is_loop_running = {
            let data_read = ctx.data.read().await;
            data_read.get::<JobRunner>().unwrap().clone()
        };

        while database.count_jobs().await.unwrap() > 0 {
            println!("loop...");
            let jobs = database.pop_executables().await.unwrap();

            for job in jobs {
                let ctx1 = Arc::clone(&ctx);
                tokio::spawn(async move {
                    println!("{:#?}", job);
                    match job.event_type {
                        EventType::Disconnect => {
                            if let Ok(_) = job.disconnect(&ctx1).await {
                                job.direct_message(&ctx1, |m| {
                                    m.content(format!("<t:{0}:d> <t:{0}:T>に通話を強制切断しました", job.timestamp()))
                                }).await.unwrap();
                            }
                        },
                        EventType::Notification3Min => {
                            let _ = job.direct_message(&ctx1, |m| {
                                m.content("3分後に通話を強制切断します")
                            }).await.unwrap();
                        }
                    }
                });
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }

        is_loop_running.store(false, Ordering::Relaxed);
    }
}

impl TypeMapKey for SqliteDatabase {
    type Value = Arc<SqliteDatabase>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let ctx = Arc::new(ctx);

        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "timezone" => commands::timezone::run(ctx.clone(), &command).await,
                "snipe" => commands::snipe::run(ctx.clone(), &command).await,
                "show" => commands::show::run(ctx.clone(), &command).await,
                "clear" => commands::clear::run(ctx.clone(), &command).await,
                _ => println!("not implemented :("),
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);

        // スラッシュコマンド登録
        let commands = Command::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::show::register(command))
                .create_application_command(|command| commands::clear::register(command))
                .create_application_command(|command| commands::snipe::register(command))
                .create_application_command(|command| commands::timezone::register(command))
        })
        .await;

        let database = {
            let data_read = ctx.data.read().await;
            data_read.get::<SqliteDatabase>().unwrap().clone()
        };
        let is_loop_running = {
            let data_read = ctx.data.read().await;
            data_read.get::<JobRunner>().unwrap().clone()
        };

        // ギルド更新
        if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
            for guild in guilds {
                if let Err(why) = database.insert_guild_setting(guild.id, 0).await
                {
                    println!("Already exists: {}, {}", guild.id, why);
                }
            }
        }

        // 起動時設定表示
        println!("create global commands: {:#?}", commands);
        if let Ok(settings) = database.get_settings().await
        {
            println!("{:#?}", settings);
        }

        if database.count_jobs().await.unwrap() > 0 {
            if let Ok(_) = is_loop_running.compare_exchange(false, true,
                                                            Ordering::Release,
                                                            Ordering::Relaxed)
            {
                JobRunner::run(Arc::new(ctx)).await;
            }
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        println!("guild_create: {:?}", guild.id);
        if is_new {
            let database = {
                let data_read = ctx.data.read().await;
                data_read.get::<SqliteDatabase>().unwrap().clone()
            };

            if let Err(why) = database.insert_guild_setting(guild.id, 0).await {
                println!("insert guild setting: {:?}", why);
            }
        }
    }

    async fn guild_delete(&self, ctx: Context, incomplete: UnavailableGuild, _: Option<Guild>) {
        println!("guild_delete {:?}", incomplete.id);
        if !incomplete.unavailable {
            let database = {
                let data_read = ctx.data.read().await;
                data_read.get::<SqliteDatabase>().unwrap().clone()
            };

            if let Err(why) = database.delete_guild_setting(incomplete.id).await {
                println!("insert guild setting: {:?}", why);
            }

        }
    }
}

#[tokio::main]
async fn main() {
    // .envファイルからDiscordトークンとIDを読み込み
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    println!("discord token: {}", token);

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected a application_id in the environment")
        .parse()
        .expect("application_id is not a valid value");

    // Botの権限
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGES;

    // データベース初期化
    let database = SqliteDatabase::new("./database.sqlite").await;

    // 指定時刻超過のジョブを削除
    if let Ok(_) = database.pop_executables().await {
        println!("Delete previous jobs.");
    }

    // クライアント初期化
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    {
        let mut data_write = client.data.write().await;
        data_write.insert::<SqliteDatabase>(Arc::new(database));
        data_write.insert::<JobRunner>(Arc::new(AtomicBool::new(false)));
    }

    // Bot起動
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
