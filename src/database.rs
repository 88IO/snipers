use chrono::NaiveDateTime;
use serenity::model::id::{UserId, GuildId};
use crate::job::{Job, EventType, GuildSetting};

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

        sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

        SqliteDatabase { database }
    }

    pub async fn pop_executables(&self) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as!(
            Job,
            r#"DELETE FROM job
               WHERE naive_utc <= CURRENT_TIMESTAMP
               RETURNING naive_utc, user_id, guild_id,
                         event_type as "event_type!: EventType""#
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
             (naive_utc, user_id, guild_id, event_type)
             SELECT $1, $2, $3, $4",
            naive_utc, user_id, guild_id, event_type
            )
            .execute(&self.database)
            .await
    }

    pub async fn delete_guild_jobs(&self,
                        user_id: UserId,
                        guild_id: GuildId)
                        -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let user_id = user_id.0 as i64;
        let guild_id = guild_id.0 as i64;

        sqlx::query!(
            "DELETE FROM job WHERE user_id=? AND guild_id=?",
            user_id, guild_id
            )
            .execute(&self.database)
            .await
    }

    pub async fn get_guild_jobs(&self, guild_id: GuildId)
                                   -> Result<Vec<Job>, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query_as!(
            Job,
            "SELECT naive_utc, user_id, guild_id,
                    event_type as 'event_type!: EventType'
             FROM job
             WHERE guild_id=?
             ORDER BY naive_utc ASC",
            guild_id)
            .fetch_all(&self.database)
            .await
    }

    pub async fn count_jobs(&self) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM job")
            .fetch_one(&self.database)
            .await?;
        Ok(result.count)
    }

    pub async fn get_guild_setting(&self, guild_id: GuildId)
                                   -> Result<GuildSetting, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query_as!(
            GuildSetting,
            "SELECT guild_id, utc_offset as 'utc_offset!: i32' FROM setting WHERE guild_id=?",
            guild_id)
            .fetch_one(&self.database)
            .await
    }

    pub async fn insert_guild_setting(&self, guild_id: GuildId, utc_offset: u64)
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

    pub async fn delete_guild_setting(&self, guild_id: GuildId)
                                -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query!(
            "DELETE FROM job WHERE guild_id=?",
            guild_id
            )
            .execute(&self.database)
            .await
    }

    pub async fn update_guild_setting(&self, guild_id: GuildId, utc_offset: i64)
                                -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let guild_id = guild_id.0 as i64;

        sqlx::query!(
            "UPDATE setting SET utc_offset=? WHERE guild_id=?",
            utc_offset, guild_id
            )
            .execute(&self.database)
            .await
    }

    pub async fn get_settings(&self) -> Result<Vec<GuildSetting>, sqlx::Error> {
        sqlx::query_as!(
            GuildSetting,
            r#"SELECT guild_id, utc_offset as "utc_offset!: i32" FROM setting"#
            )
            .fetch_all(&self.database)
            .await
    }
}
