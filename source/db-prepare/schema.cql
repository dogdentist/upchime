CREATE KEYSPACE IF NOT EXISTS upchime 
WITH replication = {
    'class': 'SimpleStrategy', 
    'replication_factor': 1
};

USE upchime;

-- stores the stuff that will be ping'd
CREATE TABLE IF NOT EXISTS target (
    target_id         UUID PRIMARY KEY,
    target_enabled    BOOLEAN,
    target_name       TEXT,
    target_address    TEXT,
    target_ping_type  TEXT,
    target_interval   INT,
    -- 0 - unknown, 1 - up, 2 - down, 3 - timeout
    target_state      TINYINT,
    target_created_at TIMESTAMP,
    target_updated_at TIMESTAMP,
    target_metadata   TEXT
);

-- stores only the ping of a target
CREATE TABLE IF NOT EXISTS ping (
    ping_target_id   UUID,
    ping_timestamp   TIMESTAMP,
    -- true means up, false means down
    ping_state       BOOLEAN,
    ping_res_time    BIGINT,
    -- only for http pings
    ping_http_status SMALLINT,

    PRIMARY KEY (ping_target_id, ping_timestamp)
) WITH CLUSTERING ORDER BY (ping_timestamp DESC);
