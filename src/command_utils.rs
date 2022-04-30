use serenity::model::interactions::application_command::{
    ApplicationCommand,
    ApplicationCommandOptionType, ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue
};
use serenity::Result;
use serenity::prelude::Context;

pub async fn register_commands(ctx: &Context) -> Result<Vec<ApplicationCommand>> {
    ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
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
            .create_application_command(|command| {
                command.name("timezone").description("UTCからの時差を設定します")
                    .create_option(|option| {
                        option
                            .name("offset")
                            .description("時差（h）")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(-12)
                            .max_int_value(12)
                            .required(true)
                    })
            })
    })
    .await
}

pub fn string_option_ref<'a>(options: &'a Vec<ApplicationCommandInteractionDataOption>, name: &str)
                             -> Option<&'a String> {
    let option_value = options
        .iter()
        .find(|&v| v.name == name)?
        .resolved
        .as_ref()?;

    if let ApplicationCommandInteractionDataOptionValue::String(s) = option_value {
        Some(s)
    } else {
        None
    }
}

pub fn int_option_ref<'a>(options: &'a Vec<ApplicationCommandInteractionDataOption>, name: &str)
                             -> Option<&'a i64> {
    let option_value = options
        .iter()
        .find(|&v| v.name == name)?
        .resolved
        .as_ref()?;

    if let ApplicationCommandInteractionDataOptionValue::Integer(i) = option_value {
        Some(i)
    } else {
        None
    }
}
