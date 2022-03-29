mod register;
mod job;
use register::register_commands;
use job::{Job, EventType};

use dotenv::dotenv;
use std::env;
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
use chrono::{Utc, Duration};
use regex::Regex;

struct Handler {
    database: sqlx::SqlitePool,
    re_time: Regex
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "snipe" => {
                    println!("{}", command.data.options.len());
                    let time = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected time option")
                        .value
                        .as_ref()
                        .expect("Expected time option");
                    let kind_option = command
                        .data
                        .options
                        .get(1);
                },
                "snipe_at" => {
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
        let commands = register_commands(&ctx).await;

        if let Ok(guilds) = ready.user.guilds(&ctx.http).await {
            for guild in guilds {
                let guild_id = guild.id.0 as i64;
                if let Err(why) = sqlx::query!(
                    "INSERT INTO setting (guild_id, utc_offset) VALUES (?, ?)",
                    guild_id, 0
                    )
                    .execute(&self.database)
                    .await
                {
                    println!("Already exists: {}, {}", guild.id, why);
                }
            }
        }

        println!("create global commands: {:#?}", commands);
    }
}

impl Handler {
    async fn snipe_in(&self, ctx: &Context, command: &ApplicationCommandInteraction) {
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

        let caps = self.re_time.captures(&interval).unwrap();
        let hour: i64 = caps["hour"].parse().unwrap();
        let minute: i64 = caps["minute"].parse().unwrap();

        let datetime = Utc::now() + Duration::hours(hour) + Duration::minutes(minute);
        let naive_utc = datetime.naive_utc();

        if let Err(why) = sqlx::query!(
            "INSERT INTO job (naive_utc, user_id, guild_id, event_type, utc_offset) SELECT ?, ?, ?, ?, utc_offset FROM setting WHERE guild_id=?",
            naive_utc, user_id, guild_id, EventType::Disconnect, guild_id
            )
            .execute(&self.database)
            .await
        {
            println!("{:?}", why);
        }

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(format!("{}を{}に切断します", Mention::from(command.user.id), datetime)))
            })
            .await
        {
            println!("cannot respond to slash command: {}", why);
        }

        self.run_pending(&ctx).await;
    }

    async fn run_pending(&self, ctx: &Context) {
        loop {
            let jobs = sqlx::query_as!(
                Job,
                r#"DELETE FROM job WHERE naive_utc <= CURRENT_TIMESTAMP RETURNING naive_utc, user_id, guild_id, event_type as "event_type!: EventType", utc_offset"#
                )
                .fetch_all(&self.database)
                .await
                .unwrap();

            for job in jobs {
                println!("{:#?}", job);
                let _ = job.guildid().disconnect_member(&ctx.http, job.userid()).await;
            }
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    println!("discord token: {}", token);

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected a application_id in the environment")
        .parse()
        .expect("application_id is not a valid value");

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(10)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true)
        )
        .await
        .expect("Couldn't connect to database");

    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

   if let Ok(_) = sqlx::query!("DELETE FROM job WHERE naive_utc <= CURRENT_TIMESTAMP")
        .execute(&database)
        .await
    {
        println!("Delete Prev Jobs.");
    }


    let handler = Handler {
        database,
        re_time: Regex::new(r"(?P<hour>\d{1,2}):(?P<minute>\d{1,2})").unwrap()
    };

    let mut client = Client::builder(token)
        .event_handler(handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    {
        //let mut data = client.data.write().await;
        //data.insert::<JobStore>(Arc::new(RwLock::new(HashMap::new())));
        //data.insert::<TimeZone>(Arc::new(RwLock::new(FixedOffset::east(0))));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
