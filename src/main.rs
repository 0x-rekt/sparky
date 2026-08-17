use tokio::net::TcpListener;

mod server;
mod resp;
mod connection;
mod commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Server listening on 127.0.0.1:6379");
    server::start_server(listener).await?;
    Ok(())
}