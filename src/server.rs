use tokio::net::TcpListener;

use crate::connection::handle_connection;
pub async fn start_server(listener: TcpListener) -> anyhow::Result<()> {
    
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tokio::spawn(async move{
            handle_connection(socket, addr).await;
        });
    }
}