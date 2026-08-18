use tokio::net::TcpListener;

use crate::{connection::handle_connection, db::SharedDb};

pub async fn start_server(listener: TcpListener, db: SharedDb) -> anyhow::Result<()> {
    
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let db_clone = db.clone();
        tokio::spawn(async move{
            handle_connection(socket, addr, db_clone).await;
        });
    }
}