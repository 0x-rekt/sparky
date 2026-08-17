use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn start_server(listener: TcpListener) -> anyhow::Result<()> {
    
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("New connection from: {}", addr);

        tokio::spawn(async move{
            let mut buf = vec![0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        println!("Connection closed by client: {}", addr);
                        break;
                    }
                    Ok(n) => {
                        let received_data = String::from_utf8_lossy(&buf[..n]);
                        println!("Received from {}: {}", addr, received_data);
                        if socket.write_all(&buf[..n]).await.is_err() {
                            eprintln!("Failed to send response to {}", addr);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket; err = {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}