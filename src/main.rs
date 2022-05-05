mod command_utils;
mod handler;
mod job;
mod database;

use handler::Handler;
use dotenv::dotenv;
use serenity::{client::Client, prelude::GatewayIntents};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    println!("discord token: {}", token);

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected a application_id in the environment")
        .parse()
        .expect("application_id is not a valid value");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_VOICE_STATES
        | GatewayIntents::DIRECT_MESSAGES;

    let handler = Handler::new("./database.sqlite").await;

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
