use serenity::{
    client::Context,
    builder::CreateMessage,
    model::{
        guild::Member,
        channel::Message,
        id::{GuildId, UserId}
    }
};
use chrono::NaiveDateTime;
use sqlx;
use std::hash::Hash;

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
}

impl Job {
    #[allow(dead_code)]
    pub fn new(naive_utc: NaiveDateTime, user_id: UserId,
               guild_id: GuildId, event_type: EventType) -> Self {
        Job { naive_utc, user_id: user_id.0 as i64, guild_id: guild_id.0 as i64, event_type }
    }

    pub fn userid(&self) -> UserId {
        UserId::from(self.user_id as u64)
    }

    pub fn guildid(&self) -> GuildId {
        GuildId::from(self.guild_id as u64)
    }

    pub fn timestamp(&self) -> i64 {
        self.naive_utc.timestamp()
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
