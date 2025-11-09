use std::collections::HashMap;

use crate::{config, db, errorln, outputln, pinger};
use futures::TryStreamExt;
use tokio::{sync::mpsc, task::AbortHandle};
use uuid::Uuid;

impl db::TargetIterObject {
    fn hash(&self) -> u64 {
        let mut digest = crc_fast::Digest::new(crc_fast::CrcAlgorithm::Crc32Cksum);

        digest.update(self.target_id.as_bytes());
        digest.update(&self.target_state.to_le_bytes());
        digest.update(self.target_name.as_bytes());
        digest.update(self.target_address.as_bytes());
        digest.update(self.target_ping_type.as_bytes());
        digest.update(&self.target_interval.to_le_bytes());
        digest.update(self.target_metadata.as_bytes());

        digest.finalize()
    }
}

pub async fn start() -> anyhow::Result<()> {
    struct TargetKvEntry {
        checksum: u64,
        sync_counter: u64,
        updater_channel: mpsc::Sender<db::TargetIterObject>,
        task_abort_handler: AbortHandle,
    }

    let config = config::object();
    // used to keep track of which sync we have done, the ones which are not
    // up to the latest sync count means they're missing from the db
    let mut sync_counter = 0;
    let mut targets_cache: Box<HashMap<Uuid, TargetKvEntry>> = Box::default();

    loop {
        sync_counter += 1;

        outputln!("synching with db; sync count: {sync_counter}");

        match db::target_iter().await {
            Ok(mut target_iter) => loop {
                match target_iter.try_next().await {
                    Ok(Some(col)) => {
                        if !col.target_enabled {
                            continue;
                        }

                        // check the target exists - if it doesn't let's create it, if
                        // it does, check for updates
                        if let Some(target) = targets_cache.get_mut(&col.target_id) {
                            let checksum = col.hash();

                            if checksum != target.checksum {
                                outputln!(
                                    "updated target; id: {}, target: '{}', type: '{}', interval: {}",
                                    col.target_id,
                                    col.target_address,
                                    col.target_ping_type,
                                    col.target_interval
                                );

                                let target_id = col.target_id;

                                if let Err(e) = target.updater_channel.send(col).await {
                                    errorln!(
                                        "failed to update the pinger of target id {target_id}, error: {}",
                                        e.to_string()
                                    );
                                }

                                target.checksum = checksum;
                                target.sync_counter = sync_counter;
                            } else {
                                target.sync_counter = sync_counter;
                            }
                        } else {
                            outputln!(
                                "new target; id: {}, target: '{}', type: '{}', interval: {}",
                                col.target_id,
                                col.target_address,
                                col.target_ping_type,
                                col.target_interval
                            );

                            let target_id = col.target_id;
                            let target_checksum = col.hash();
                            let (update_channel_tx, update_channel_rx) =
                                mpsc::channel::<db::TargetIterObject>(1);
                            let task_abort_handler =
                                tokio::spawn(pinger::ping(col, update_channel_rx)).abort_handle();

                            targets_cache.insert(
                                target_id,
                                TargetKvEntry {
                                    checksum: target_checksum,
                                    sync_counter,
                                    updater_channel: update_channel_tx,
                                    task_abort_handler,
                                },
                            );
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        errorln!("syncing with the db failed, error: {}", e.to_string());
                        break;
                    }
                }
            },
            Err(e) => errorln!("failed to sync with the db, error: {}", e.to_string()),
        }

        targets_cache.retain(|target_id, v| {
            if v.sync_counter != sync_counter {
                outputln!("removing target with id: {target_id}");

                v.task_abort_handler.abort();

                false
            } else {
                true
            }
        });

        outputln!("synching completed, total targets: {}", targets_cache.len());
        tokio::time::sleep(tokio::time::Duration::from_secs(config.db_sync_interval)).await;
    }
}
