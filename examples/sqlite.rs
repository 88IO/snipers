use sqlx;
use serenity::model::id::{UserId, GuildId};
use chrono::{Utc, NaiveDateTime};

#[derive(Debug, sqlx::Type)]
enum EventType {
    EventA,
    EventB
}

#[derive(Debug)]
struct User {
    pub naive_utc: NaiveDateTime,
    pub user_id: i64,
    pub guild_id: i64,
    pub event_type: EventType,
    pub utc_offset: i64
}


#[tokio::main]
async fn main() {
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
    println!("migration success");

    let now = Utc::now().naive_utc();
    let now = NaiveDateTime::from_timestamp(10000,100);
    /*
    let entry = sqlx::query_as!(
        User,
        "SELECT * FROM job"
        )
        .fetch_all(&database)
        .await
        .unwrap();
    */

    let entry = sqlx::query_as!(
        User,
        r#"DELETE FROM job WHERE naive_utc >= CURRENT_TIMESTAMP RETURNING naive_utc, user_id, guild_id, event_type as "event_type!: EventType", utc_offset"#
        )
        .fetch_all(&database)
        .await
        .unwrap();

    println!("{:?}", entry);
}
