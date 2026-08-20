use tokio::net::TcpListener;

mod commands;
mod connection;
mod db;
mod persistence;
mod resp;
mod server;

fn print_startup_banner() {
    const CYAN: &str = "\x1b[36m";
    const GREEN: &str = "\x1b[32m";
    const BOLD: &str = "\x1b[1m";
    const DIM: &str = "\x1b[2m";
    const RESET: &str = "\x1b[0m";

    println!();
    println!("{CYAN}{BOLD} ____  ____   _    ____  _  __ __   __{RESET}");
    println!("{CYAN}{BOLD}/ ___||  _ \\ / \\  |  _ \\| |/ / \\ \\ / /{RESET}");
    println!("{CYAN}{BOLD}\\___ \\| |_) / _ \\ | |_) | ' /   \\ V / {RESET}");
    println!("{CYAN}{BOLD} ___) |  __/ ___ \\|  _ <| . \\    | |  {RESET}");
    println!("{CYAN}{BOLD}|____/|_| /_/   \\_\\_| \\_\\_|\\_\\   |_|  {RESET}");
    println!();
    println!("{GREEN}{BOLD}           SPARKY SERVER IS LIVE{RESET}");
    println!("{DIM}-------------------------------------------------{RESET}");
    println!();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    print_startup_banner();
    let db = db::create_shared_db();
    let aof_path = std::env::var("SPARKY_AOF").unwrap_or_else(|_| "sparky.aof".to_string());
    let aof = persistence::aof::Aof::open(&aof_path)?;
    persistence::aof::Aof::replay(&aof_path, db.clone())?;
    let port = std::env::var("SPARKY_PORT").unwrap_or_else(|_| "6969".to_string());
    let address = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&address).await?;
    println!("Server listening on {address}");
    db::spawn_expiry_cleaner(db.clone());
    server::start_server(listener, db, aof).await?;
    Ok(())
}
