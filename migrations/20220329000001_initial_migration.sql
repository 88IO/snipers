CREATE TABLE IF NOT EXISTS job (
    naive_utc DATETIME NOT NULL,
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    event_type INTEGER NOT NULL,
    utc_offset INTEGER NOT NULL,
    UNIQUE(naive_utc, user_id, guild_id, event_type)
);

CREATE TABLE IF NOT EXISTS setting (
    guild_id BIGINT NOT NULL PRIMARY KEY,
    utc_offset INTEGER NOT NULL
);

