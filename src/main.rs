use dotenv::dotenv;
use std::env;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{
            application_command::{
                ApplicationCommand,
                ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction,
            InteractionResponseType,
        },
    },
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
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
                    if let Some(kind) = kind_option {
                        format!("{}: {}", time, kind.value.as_ref().unwrap())
                    } else {
                        format!("{}: no kind", time)
                    }
                },
                "snipe_at" => {
                    let time = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected time option")
                        .value
                        .as_ref()
                        .expect("Expected time option")
                        .to_string();
                    time
                },
                "snipe_in" => {
                    let interval = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected time option")
                        .value
                        .as_ref()
                        .expect("Expected time option")
                        .to_string();
                    interval
                },
                "id" => {
                    let options = command
                        .data
                        .options
                        .get(0)
                        .expect("Expected user option")
                        .resolved
                        .as_ref()
                        .expect("Expected user object");

                    if let ApplicationCommandInteractionDataOptionValue::User(user, _member) =
                        options
                    {
                        format!("{}'s id is {}", user.tag(), user.id)
                    } else {
                        "Please provide a valid user".to_string()
                    }
                },
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);

        let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
            commands
                .create_application_command(|command| {
                    command.name("snipe_at")
                        .description("指定した時刻に通話を強制切断します")
                        .create_option(|option| {
                            option
                                .name("time")
                                .description("切断する時刻")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("snipe_in")
                        .description("指定した時間後に通話を強制切断します")
                        .create_option(|option| {
                            option
                                .name("interval")
                                .description("切断するまでの時間")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                })
                .create_application_command(|command| {
                    command.name("schedule").description("通話の切断予定を表示します")
                })
                .create_application_command(|command| {
                    command.name("clear").description("通話の切断予定を削除します")
                })
                .create_application_command(|command| {
                    command.name("snipe").description("通話の切断予定を削除します")
                        .create_option(|option| {
                            option
                                .name("time")
                                .description("切断する時刻")
                                .kind(ApplicationCommandOptionType::String)
                                .required(true)
                        })
                        .create_option(|option| {
                            option
                                .name("kind")
                                .description("指定方法を選択します")
                                .kind(ApplicationCommandOptionType::String)
                                .add_string_choice("oclock", "oclock")
                                .add_string_choice("after", "after")
                        })
                })
        })
        .await;

        println!("create global commands: {:#?}", commands);
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

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client.");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
