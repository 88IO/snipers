mod command_utils;
mod handler;
mod job;
mod database;

use handler::Handler;
use dotenv::dotenv;
use serenity::Client;
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

    //sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");


    let handler = Handler::new("./database.sqlite").await;
    let mut client = Client::builder(token)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
