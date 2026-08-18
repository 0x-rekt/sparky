use tokio::net::TcpListener;

mod server;
mod resp;
mod connection;
mod commands;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();
    println!("Server listening on 127.0.0.1:6969");
    let db = db::create_shared_db();
    server::start_server(listener, db).await?;
    Ok(())
}