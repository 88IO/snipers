use serenity::{model::{id::{GuildId, UserId}, guild::Member, channel::Message}, client::Context, builder::CreateMessage};
use std::hash::Hash;
use chrono::{NaiveDateTime, DateTime, FixedOffset, TimeZone};
use sqlx;

#[derive(Debug, Hash, PartialEq, sqlx::Type)]
pub enum EventType {
    Disconnect,
    Notification3Min,
}

#[derive(Debug)]
pub struct GuildSetting {
    pub guild_id: i64,
    pub utc_offset: i32
}

#[derive(Debug)]
pub struct Job {
    pub naive_utc: NaiveDateTime,
    pub user_id: i64,
    pub guild_id: i64,
    pub event_type: EventType,
    pub utc_offset: i32
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
    pub fn new(naive_utc: NaiveDateTime, user_id: UserId, guild_id: GuildId, event_type: EventType, utc_offset: i32) -> Self {
        Job { naive_utc, user_id: user_id.0 as i64, guild_id: guild_id.0 as i64, event_type, utc_offset }
    }

    pub fn userid(&self) -> UserId {
        UserId::from(self.user_id as u64)
    }

    pub fn guildid(&self) -> GuildId {
        GuildId::from(self.guild_id as u64)
    }

    pub fn datetime(&self) -> DateTime<FixedOffset> {
        FixedOffset::east(3600 * self.utc_offset)
            .from_utc_datetime(&self.naive_utc)
    }

    pub async fn disconnect(&self, ctx: &Context) -> serenity::Result<Member> {
        self.guildid().disconnect_member(&ctx.http, self.userid()).await
    }

    pub async fn direct_message<'a, F>(&self, ctx: &Context, f: F) -> serenity::Result<Message>
    where
        for<'b> F: FnOnce(&'b mut CreateMessage<'a>) -> &'b mut CreateMessage<'a>
    {
        self.userid()
            .create_dm_channel(&ctx.http).await?
            .send_message(&ctx.http, f).await
    }
}
