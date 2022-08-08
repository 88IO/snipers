mod command_utils;
mod handler;
mod job;
mod database;

use handler::Handler;
use regex::Regex;
use database::SqliteDatabase;
use dotenv::dotenv;
use serenity::{client::Client, prelude::GatewayIntents};
use std::env;

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
    // 時刻指定のパーサー
    let re_time = Regex::new(r"(?P<hour>\d{1,2}):(?P<minute>\d{1,2})").unwrap();

    // 指定時刻超過のジョブを削除
    if let Ok(_) = database.pull_executables().await {
        println!("Delete previous jobs.");
    }

    // イベントハンドラー初期化
    let handler = Handler::new(database, re_time).await;

    // クライアント初期化
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    // Bot起動
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
