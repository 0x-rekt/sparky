use std::sync::{Arc, Mutex};

use tokio::net::TcpListener;

use crate::{connection::handle_connection, db::SharedDb, persistence::aof::Aof};

pub async fn start_server(listener: TcpListener, db: SharedDb, aof: Aof) -> anyhow::Result<()> {
    let command_lock = Arc::new(Mutex::new(()));
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let db_clone = db.clone();
        let aof_clone = aof.clone();
        let command_lock_clone = command_lock.clone();
        tokio::spawn(async move {
            handle_connection(socket, addr, db_clone, aof_clone, command_lock_clone).await;
        });
    }
}
