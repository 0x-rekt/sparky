use tokio::net::TcpListener;

mod server;
mod resp;
mod connection;
mod commands;
mod db;

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
    let listener = TcpListener::bind("127.0.0.1:6969").await.unwrap();
    println!("Server listening on 127.0.0.1:6969");
    let db = db::create_shared_db();
    db::spawn_expiry_cleaner(db.clone());
    server::start_server(listener, db).await?;
    Ok(())
}