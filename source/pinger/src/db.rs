use scylla::{
    DeserializeRow,
    client::{pager::TypedRowStream, session::Session, session_builder::SessionBuilder},
    statement::prepared::PreparedStatement,
};
use uuid::Uuid;

use crate::{config, constants};

async fn open_conn() -> anyhow::Result<Session> {
    let config = config::object();

    Ok(SessionBuilder::new()
        .known_node(constants::DB_FQDN.to_owned() + ":9042")
        .use_keyspace(constants::DB_KEYSPACE, false)
        .user(config.db_username.to_owned(), config.db_password.to_owned())
        .build()
        .await?)
}

#[repr(i8)]
pub enum TargetState {
    Unknown = 0,
    Up = 1,
    Down = 2,
    Timeout = 3,
}

#[derive(Debug, DeserializeRow)]
pub struct TargetIterObject {
    pub target_id: Uuid,
    pub target_enabled: bool,
    pub target_name: String,
    pub target_address: String,
    pub target_ping_type: String,
    pub target_interval: i32,
    pub target_state: i8,
    pub target_metadata: String,
}

pub async fn target_iter() -> anyhow::Result<TypedRowStream<TargetIterObject>> {
    let session = open_conn().await?;

    let query_pager = session
        .query_iter(
            r#"SELECT
                target_id,
                target_enabled,
                target_name,
                target_address,
                target_ping_type,
                target_interval,
                target_state,
                target_metadata
            FROM target
        "#,
            &[],
        )
        .await?;

    Ok(query_pager.rows_stream::<TargetIterObject>()?)
}

pub async fn update_target_state(id: Uuid, state: TargetState) -> anyhow::Result<()> {
    let session = open_conn().await?;

    let prepared: PreparedStatement = session
        .prepare(
            r#"
                UPDATE target
                SET target_state = ?
                WHERE target_id = ?
            "#,
        )
        .await?;

    session
        .execute_unpaged(&prepared, (state as i8, id))
        .await?;

    Ok(())
}

pub async fn inser_http_ping(
    id: Uuid,
    up: bool,
    response_time: Option<u64>,
    status: Option<u16>,
) -> anyhow::Result<()> {
    let session = open_conn().await?;

    let prepared: PreparedStatement = session
        .prepare(
            r#"
                INSERT INTO ping (
                    ping_target_id,
                    ping_timestamp,
                    ping_state,
                    ping_res_time,
                    ping_http_status
                ) VALUES (
                    ?, ?, ?, ?, ?
                )
            "#,
        )
        .await?;

    session
        .execute_unpaged(
            &prepared,
            (
                id,
                chrono::Utc::now(),
                up,
                response_time.map(|v| v as i64),
                status.map(|v| v as i16),
            ),
        )
        .await?;

    Ok(())
}
