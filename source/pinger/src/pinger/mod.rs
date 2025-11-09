use anyhow::anyhow;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{db, errorln, outputln};

mod http;

async fn http_ping_wrapper(
    id: Uuid,
    timeout: u64,
    current_state: &mut i8,
    target_address: &str,
    metadata: &http::HttpMetadata,
) -> anyhow::Result<()> {
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(timeout),
        http::ping(target_address, metadata),
    )
    .await
    {
        Ok(res) => match res {
            Ok(http::PingResponse::Up((response_time, status_code))) => {
                if *current_state != db::TargetState::Up as i8 {
                    outputln!("target with id {id} is UP, status code: {status_code}",);

                    *current_state = db::TargetState::Up as i8;
                    db::update_target_state(id, db::TargetState::Down).await?
                }

                db::inser_http_ping(id, true, Some(response_time), Some(status_code)).await?;
            }
            Ok(http::PingResponse::Down((response_time, status_code))) => {
                if *current_state != db::TargetState::Down as i8 {
                    errorln!("target with id {id} is DOWN, status code: {status_code}",);

                    *current_state = db::TargetState::Down as i8;
                    db::update_target_state(id, db::TargetState::Down).await?
                }

                db::inser_http_ping(id, false, Some(response_time), Some(status_code)).await?;
            }
            Ok(http::PingResponse::Timeout) => {
                if *current_state != db::TargetState::Timeout as i8 {
                    errorln!("ping on target with id {id} timeout'd");

                    *current_state = db::TargetState::Timeout as i8;
                    db::update_target_state(id, db::TargetState::Timeout).await?
                }

                db::inser_http_ping(id, false, None, None).await?;
            }
            Err(e) => {
                if *current_state != db::TargetState::Down as i8 {
                    errorln!(
                        "ping on target with id {id} failed with error: {}",
                        e.to_string()
                    );

                    *current_state = db::TargetState::Down as i8;
                    db::update_target_state(id, db::TargetState::Down).await?
                }

                db::inser_http_ping(id, false, None, None).await?;
            }
        },
        Err(_) => {
            if *current_state != db::TargetState::Timeout as i8 {
                errorln!("ping on target with id {id} timeout'd");

                *current_state = db::TargetState::Timeout as i8;
                db::update_target_state(id, db::TargetState::Timeout).await?
            }

            db::inser_http_ping(id, false, None, None).await?;
        }
    }

    Ok(())
}

pub async fn ping(
    target: db::TargetIterObject,
    mut update_channel: mpsc::Receiver<db::TargetIterObject>,
) -> anyhow::Result<()> {
    let id = target.target_id;
    let mut target = target;

    // changing of target's ping type shouldn't be allowed, the user must be forced to
    // create a new target
    match target.target_ping_type.as_str() {
        "HTTP" => {
            let mut current_state = target.target_state;

            loop {
                let metadata: http::HttpMetadata = serde_json::from_str(&target.target_metadata)
                    .map_err(|e| anyhow!("invalid metadata, {}", e))?;

                loop {
                    if let Ok(new_target) = update_channel.try_recv() {
                        target = new_target;
                        current_state = target.target_state;
                        break;
                    }

                    let ping_start = chrono::Utc::now().timestamp_millis() as u64;

                    if let Err(e) = http_ping_wrapper(
                        id,
                        target.target_interval as u64,
                        &mut current_state,
                        &target.target_address,
                        &metadata,
                    )
                    .await
                    {
                        errorln!(
                            "pinging target with {id} failed with error: {}",
                            e.to_string()
                        );
                    }

                    let ping_duration = chrono::Utc::now().timestamp_millis() as u64 - ping_start;
                    let interval = target.target_interval as u64 * 1000;

                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        if ping_duration >= interval {
                            // overloaded ping
                            0
                        } else {
                            interval.saturating_sub(ping_duration)
                        },
                    ))
                    .await;
                }
            }
        }
        _ => Err(anyhow!("invalid ping type: '{}'", target.target_ping_type)),
    }
}
