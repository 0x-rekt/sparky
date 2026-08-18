use bytes::BytesMut;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::commands::handle_command;
use crate::db::SharedDb;
use crate::resp::parser::parse_message;
use crate::resp::serializer;

pub async fn handle_connection(mut socket: TcpStream, addr: SocketAddr, db: SharedDb) {
    println!("Handling connection from: {}", addr);
    let mut buffer = BytesMut::with_capacity(1024);
    loop {
        let mut chunk = vec![0; 1024];
        match socket.read(&mut chunk).await {
            Ok(0) => {
                println!("Connection closed by client: {}", addr);
                break;
            }
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(e) => {
                eprintln!("Failed to read from socket; err = {:?}", e);
                break;
            }
        }

        while let Ok((request, consumed)) = parse_message(&buffer) {
            let response = handle_command(request, db.clone());
            let serialized_response = serializer::serialize(&response);
            if socket.write_all(&serialized_response).await.is_err() {
                eprintln!("Failed to write to socket");
                break;
            }
            let _ = buffer.split_to(consumed);
        }
    }
}
