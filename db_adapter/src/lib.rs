use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use serde::{Deserialize, Serialize};

use nexosim::model::{Context, Model, ProtoModel};

const QUEUE_SIZE: usize = 256;
const RECONNECT_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);

/// Basic db adapter.
///
/// When the db is not available it fails silently (to allow testing).
/// Sync db client is used as the async one depends on tokio runtime.
#[derive(Serialize, Deserialize)]
pub struct DbAdapter;

#[Model(type Env=DbEnv)]
impl DbAdapter {
    /// This method should not be used on untrusted sources
    /// as it's prone to SQL injection attacks.
    pub fn execute_query(&mut self, stmt: String, _: &Context<Self>, env: &mut DbEnv) {
        if let Some(tx) = &env.tx {
            // Send the query to the client thread.
            let _ = tx.try_send(stmt);
        }
    }
}

pub struct DbEnv {
    tx: Option<SyncSender<String>>,
    handle: Option<std::thread::JoinHandle<()>>,
}
impl Drop for DbEnv {
    fn drop(&mut self) {
        // Drop sender to disconnect the channel.
        self.tx = None;
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }
}

pub struct ProtoDb {
    connection_str: String,
}
impl ProtoDb {
    pub fn new(connection_str: String) -> Self {
        Self { connection_str }
    }
}
impl ProtoModel for ProtoDb {
    type Model = DbAdapter;

    fn build(
        self,
        _: &mut nexosim::model::BuildContext<Self>,
    ) -> (Self::Model, <Self::Model as nexosim::model::Model>::Env) {
        let (tx, rx) = sync_channel(QUEUE_SIZE);
        let handle = std::thread::spawn(move || {
            db_thread(rx, self.connection_str);
        });

        (
            DbAdapter,
            DbEnv {
                tx: Some(tx),
                handle: Some(handle),
            },
        )
    }
}

fn db_thread(rx: Receiver<String>, connection_str: String) {
    let mut client = None;

    let mut last_reconnect = std::time::Instant::now() - RECONNECT_INTERVAL;

    while let Ok(stmt) = rx.recv() {
        if client.is_none() {
            if last_reconnect.elapsed() < RECONNECT_INTERVAL {
                continue;
            }

            // Connect the client.
            match postgres::Client::connect(&connection_str, postgres::NoTls) {
                Err(e) => {
                    tracing::error!("Db connection has failed: {e}");
                    last_reconnect = std::time::Instant::now();
                    continue;
                }
                Ok(c) => {
                    client = Some(c);
                    tracing::info!("Successfully connected to the Db")
                }
            }
        }

        tracing::debug!("Executing db query {stmt}");
        if let Err(e) = client.as_mut().unwrap().execute(&stmt, &[]) {
            tracing::error!("Db query has failed: {e}");
            // Check for a closed connection. If closed drop the client.
            if e.is_closed() {
                client = None;
            }
        };
    }
}
