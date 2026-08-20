use std::time::Duration;

use tokio::sync::{mpsc, oneshot};

use crate::commands::handle_command;
use crate::db::Db;
use crate::persistence::aof::{Aof, is_write_command};
use crate::resp::RespValue;

pub struct Command {
    pub request: RespValue,
    pub response_sender: oneshot::Sender<RespValue>,
}

#[derive(Clone)]
pub struct DbHandle {
    pub tx: mpsc::Sender<Command>,
}

pub async fn run_actor(mut rx: mpsc::Receiver<Command>, mut db: Db, aof: Aof) {
    let mut expiry_tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                let response = handle_command(cmd.request.clone(), &mut db);
                if is_write_command(&cmd.request)
                    && !matches!(response, RespValue::Error(_))
                    && let Err(error) = aof.append(&cmd.request)
                {
                    eprintln!("AOF append failed: {error}");
                }
                let _ = cmd.response_sender.send(response);
            }
            _ = expiry_tick.tick() => {
                db.clear_expired_keys();
                if let Err(error) = aof.sync_if_due() {
                    eprintln!("AOF sync failed: {error}");
                }
            },
            else => break,
        }
    }
}

impl DbHandle {
    pub fn new(tx: mpsc::Sender<Command>) -> Self {
        Self { tx }
    }

    pub async fn send_command(&self, request: RespValue) -> RespValue {
        let (response_sender, response_receiver) = oneshot::channel();
        let command = Command {
            request,
            response_sender,
        };
        if self.tx.send(command).await.is_err() {
            return RespValue::Error("ERR database actor has shut down".to_string());
        }
        match response_receiver.await {
            Ok(response) => response,
            Err(_) => {
                RespValue::Error("ERR failed to receive response from database actor".to_string())
            }
        }
    }
}
