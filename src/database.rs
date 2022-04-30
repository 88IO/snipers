use crate::job::{Job, EventType, GuildSetting};
use chrono::NaiveDateTime;
use serenity::model::id::{UserId, GuildId};

pub struct SqliteDatabase {
    database: sqlx::SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(filename: &str) -> Self {
        let database = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(filename)
                    .create_if_missing(true)
            )
            .await
            .expect("Couldn't connect to database");

        SqliteDatabase { database }
    }

    pub async fn pull_executables(&self) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as!(
            Job,
            r#"DELETE FROM job
               WHERE naive_utc <= CURRENT_TIMESTAMP
               RETURNING naive_utc, user_id, guild_id,
                         event_type as "event_type!: EventType",
                         utc_offset as "utc_offset!: i32""#
            )
            .fetch_all(&self.database)
            .await
    }

    pub async fn insert_job(&self,
                        naive_utc: NaiveDateTime,
                        user_id: UserId,
                        guild_id: GuildId,
                        event_type: EventType)
                        -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let user_id = user_id.0 as i64;
        let guild_id = guild_id.0 as i64;

        sqlx::query!(
            "INSERT INTO job
             (naive_utc, user_id, guild_id, event_type, utc_offset)
             SELECT $1, $2, $3, $4, utc_offset FROM setting WHERE guild_id=$3",
            naive_utc, user_id, guild_id, event_type
            )
            .execute(&self.database)
            .await
    }

    pub async fn count_jobs(&self) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!("SELECT COUNT(*) as count from job")
            .fetch_one(&self.database)
            .await?;
        Ok(result.count)
    }

    pub async fn get_guild_setting(&self, guild_id: GuildId)
                                   -> Result<GuildSetting, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query_as!(
            GuildSetting,
            "SELECT guild_id, utc_offset as 'utc_offset!: i32' from setting WHERE guild_id=?",
            guild_id)
            .fetch_one(&self.database)
            .await
    }

    pub async fn get_settings(&self) -> Result<Vec<GuildSetting>, sqlx::Error> {
        sqlx::query_as!(
            GuildSetting,
            r#"SELECT guild_id, utc_offset as "utc_offset!: i32" from setting"#
            )
            .fetch_all(&self.database)
            .await
    }

    pub async fn insert_setting(&self, guild_id: GuildId, utc_offset: u64)
                                -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let guild_id = guild_id.0 as i64;
        let utc_offset = utc_offset as i64;

        sqlx::query!(
            "INSERT INTO setting (guild_id, utc_offset) VALUES (?, ?)",
            guild_id, utc_offset
            )
            .execute(&self.database)
            .await
    }

    pub async fn update_setting(&self, guild_id: GuildId, utc_offset: &i64)
                                -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query!(
            "UPDATE setting SET utc_offset = ? WHERE guild_id = ?",
            utc_offset, guild_id
            )
            .execute(&self.database)
            .await
    }
}
