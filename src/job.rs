use serenity::model::id::{GuildId, UserId};
use std::hash::{Hash, Hasher};
use chrono::NaiveDateTime;
use sqlx;

#[derive(Debug, Hash, PartialEq, sqlx::Type)]
pub enum EventType {
    Disconnect,
    Notification3Min,
}

#[derive(Debug)]
pub struct Job {
    pub naive_utc: NaiveDateTime,
    pub user_id: i64,
    pub guild_id: i64,
    pub event_type: EventType,
    pub utc_offset: i64
}

/*
impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.user_id.hash(state);
        self.guild_id.hash(state);
        self.event_type.hash(state);
    }
}
impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
            && self.guild_id == other.guild_id
            && self.event_type == other.event_type
    }
}
impl Eq for Job {}
*/

impl Job {
    pub fn new(naive_utc: NaiveDateTime, user_id: UserId, guild_id: GuildId, event_type: EventType, utc_offset: i64) -> Self {
        Job { naive_utc, user_id: user_id.0 as i64, guild_id: guild_id.0 as i64, event_type, utc_offset }
    }
    pub fn userid(&self) -> UserId {
        UserId::from(self.user_id as u64)
    }
    pub fn guildid(&self) -> GuildId {
        GuildId::from(self.guild_id as u64)
    }
}
