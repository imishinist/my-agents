use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::broadcast;

use crate::router::SseEvent;

/// Heartbeat monitor that periodically checks for stalled Workers.
pub struct HeartbeatMonitor {
    pool: SqlitePool,
    event_tx: broadcast::Sender<SseEvent>,
    interval: Duration,
    timeout_minutes: i64,
}

impl HeartbeatMonitor {
    pub fn new(
        pool: SqlitePool,
        event_tx: broadcast::Sender<SseEvent>,
        interval: Duration,
        timeout_minutes: i64,
    ) -> Self {
        Self {
            pool,
            event_tx,
            interval,
            timeout_minutes,
        }
    }

    /// Run the heartbeat monitor loop. Call this in a tokio::spawn.
    pub async fn run(self) {
        let mut ticker = tokio::time::interval(self.interval);

        loop {
            ticker.tick().await;

            match conducer_state::queries::get_stalled_workers(&self.pool, self.timeout_minutes)
                .await
            {
                Ok(stalled) => {
                    for worker in &stalled {
                        tracing::warn!(
                            worker_id = %worker.id,
                            last_heartbeat = ?worker.last_heartbeat,
                            "Worker stalled"
                        );

                        // Mark worker as stalled
                        let _ = conducer_state::queries::update_worker_status(
                            &self.pool,
                            &worker.id,
                            "stalled",
                        )
                        .await;

                        let _ = self.event_tx.send(SseEvent {
                            event_type: "worker.stalled".to_string(),
                            data: serde_json::json!({
                                "worker_id": worker.id,
                                "last_heartbeat": worker.last_heartbeat,
                            })
                            .to_string(),
                        });
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to check stalled workers");
                }
            }
        }
    }
}
