use serenity::builder::{CreateButton, CreateActionRow};
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption,
    CommandDataOptionValue,
};
use serenity::model::application::component::ButtonStyle;
use std::fmt;

pub const DT_FORMAT: &str = "%m/%d %H:%M:%S (%:z)";

pub enum SnipeType {
    Relative,
    Absolute
}

impl fmt::Display for SnipeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
         match self {
             Self::Relative => write!(f, "時間後"),
             Self::Absolute => write!(f, "時刻"),
         }
    }
}

impl SnipeType {
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

    fn custom_id(&self) -> &str {
        match self {
            Self::Relative => "in",
            Self::Absolute => "at",
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.custom_id());
        b.emoji(self.emoji());
        b.label(self);
        b.style(self.style());
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        ar.add_button(SnipeType::Absolute.button());
        ar.add_button(SnipeType::Relative.button());
        ar
    }
}

pub fn string_option_ref<'a>(options: &'a [CommandDataOption], name: &str)
                             -> Option<&'a String> {
    let option_value = options
        .iter()
        .find(|&v| v.name == name)?
        .resolved
        .as_ref()?;

    if let CommandDataOptionValue::String(s) = option_value {
        Some(s)
    } else {
        None
    }
}

pub fn int_option_ref<'a>(options: &'a [CommandDataOption], name: &str)
                             -> Option<&'a i64> {
    let option_value = options
        .iter()
        .find(|&v| v.name == name)?
        .resolved
        .as_ref()?;

    if let CommandDataOptionValue::Integer(i) = option_value {
        Some(i)
    } else {
        None
    }
}
