use serenity::builder::{CreateButton, CreateActionRow};
use serenity::model::interactions::application_command::{
    ApplicationCommand,
    ApplicationCommandOptionType, ApplicationCommandInteractionDataOption,
    ApplicationCommandInteractionDataOptionValue
};
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::prelude::Context;
use std::fmt;
#[derive(Debug)]
pub struct ParseComponentError(String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl std::error::Error for ParseComponentError {}

impl std::str::FromStr for SnipeMenu {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "時間後" => Ok(SnipeMenu::Relative),
            "時刻" => Ok(SnipeMenu::Absolute),
            _ => Err(ParseComponentError(s.to_string()))
        }
    }
}

pub enum SnipeMenu {
    Relative,
    Absolute
}

impl fmt::Display for SnipeMenu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
             Self::Relative => write!(f, "時間後"),
             Self::Absolute => write!(f, "時刻"),
         }
    }
}

impl SnipeMenu {
    fn emoji(&self) -> char {
        match self {
            Self::Relative => '\u{23F2}',
            Self::Absolute => '\u{23F0}',
        }
    }

    fn style(&self) -> ButtonStyle {
        match self {
            Self::Relative => ButtonStyle::Primary,
            Self::Absolute => ButtonStyle::Secondary,
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string());
        b.emoji(self.emoji());
        b.label(self);
        b.style(self.style());
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(SnipeMenu::Absolute.button());
        ar.add_button(SnipeMenu::Relative.button());
        ar
    }
}

pub async fn register_commands(ctx: &Context) -> serenity::Result<Vec<ApplicationCommand>> {
    ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
        commands
            /*
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
            */
            .create_application_command(|command| {
                command.name("display").description("通話の切断予定を表示します")
            })
            .create_application_command(|command| {
                command.name("clear").description("通話の切断予定を削除します")
            })
            .create_application_command(|command| {
                command.name("snipe").description("通話の切断予定を削除します")
                    .create_option(|option| {
                        option
                            .name("time")
                            .description("切断する絶対/相対時刻")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
                    .create_option(|option| {
                        option
                            .name("kind")
                            .description("指定方法を選択します")
                            .kind(ApplicationCommandOptionType::String)
                            .add_string_choice("o'clock", "oclock")
                            .add_string_choice("later", "later")
                    })
            })
            .create_application_command(|command| {
                command.name("timezone").description("UTCからの時差を設定/表示します")
                    .create_option(|option| {
                        option
                            .name("offset")
                            .description("時差（h）")
                            .kind(ApplicationCommandOptionType::Integer)
                            .min_int_value(-12)
                            .max_int_value(12)
                            .required(false)
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
