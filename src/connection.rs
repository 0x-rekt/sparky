use bytes::BytesMut;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::db::actor::DbHandle;
use crate::resp::parser::parse_message;
use crate::resp::serializer;

pub async fn handle_connection(mut socket: TcpStream, addr: SocketAddr, db: DbHandle) {
    let debug_connections = std::env::var("SPARKY_DEBUG_CONNECTIONS").is_ok();
    if debug_connections {
        println!("Handling connection from: {addr}");
    }
    let mut buffer = BytesMut::with_capacity(4096);
    let mut chunk = [0_u8; 4096];
    let mut write_buffer = Vec::with_capacity(256);

    loop {
        match socket.read(&mut chunk).await {
            Ok(0) => {
                if debug_connections {
                    println!("Connection closed by client: {addr}");
                }
                break;
            }
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(error) => {
                eprintln!("Failed to read from socket; err = {error:?}");
                break;
            }
        }

        while let Ok((request, consumed)) = parse_message(&buffer) {
            let response = db.send_command(request).await;
            write_buffer.clear();
            serializer::serialize_into(&response, &mut write_buffer);
            if socket.write_all(&write_buffer).await.is_err() {
                eprintln!("Failed to write to socket");
                break;
            }
            let _ = buffer.split_to(consumed);
        }
    }
}
