use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind ephemeral port");
    listener.local_addr().unwrap().port()
}

fn aof_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sparky-integration-{timestamp}.aof"))
}

fn start_server(port: u16, aof: &PathBuf) -> Child {
    let child = Command::new(env!("CARGO_BIN_EXE_sparky"))
        .env("SPARKY_PORT", port.to_string())
        .env("SPARKY_AOF", aof)
        .spawn()
        .expect("start Sparky");

    for _ in 0..50 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            return child;
        }
        thread::sleep(Duration::from_millis(50));
    }

    let mut child = child;
    let _ = child.kill();
    let _ = child.wait();
    panic!("Sparky did not start on port {port}");
}

fn stop_server(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn command(parts: &[&str]) -> Vec<u8> {
    let mut request = format!("*{}\r\n", parts.len()).into_bytes();
    for part in parts {
        request.extend_from_slice(format!("${}\r\n", part.len()).as_bytes());
        request.extend_from_slice(part.as_bytes());
        request.extend_from_slice(b"\r\n");
    }
    request
}

fn send(stream: &mut TcpStream, parts: &[&str]) -> String {
    stream.write_all(&command(parts)).unwrap();
    stream.flush().unwrap();

    let mut response = Vec::new();
    let mut byte = [0u8; 1];
    let mut line = Vec::new();
    stream.read_exact(&mut byte).unwrap();
    response.push(byte[0]);

    match byte[0] {
        b'+' | b'-' | b':' => {
            read_line(stream, &mut line, &mut response);
        }
        b'$' => {
            read_line(stream, &mut line, &mut response);
            let length: usize = String::from_utf8_lossy(&line).parse().unwrap();
            if length != usize::MAX {
                let mut payload = vec![0; length + 2];
                stream.read_exact(&mut payload).unwrap();
                response.extend_from_slice(&payload);
            }
        }
        b'*' => {
            read_line(stream, &mut line, &mut response);
            let count: usize = String::from_utf8_lossy(&line).parse().unwrap();
            for _ in 0..count {
                let mut nested = [0u8; 1];
                stream.read_exact(&mut nested).unwrap();
                response.push(nested[0]);
                assert_eq!(nested[0], b'$');
                line.clear();
                read_line(stream, &mut line, &mut response);
                let length: usize = String::from_utf8_lossy(&line).parse().unwrap();
                let mut payload = vec![0; length + 2];
                stream.read_exact(&mut payload).unwrap();
                response.extend_from_slice(&payload);
            }
        }
        other => panic!("unexpected RESP prefix: {other:?}"),
    }

    String::from_utf8(response).unwrap()
}

fn read_line(stream: &mut TcpStream, line: &mut Vec<u8>, response: &mut Vec<u8>) {
    let mut byte = [0u8; 1];
    loop {
        stream.read_exact(&mut byte).unwrap();
        response.push(byte[0]);
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
    }
    if line.ends_with(b"\r") {
        line.pop();
    }
}

#[test]
fn commands_and_aof_replay_survive_restart() {
    let port = free_port();
    let aof = aof_path();

    let mut server = start_server(port, &aof);
    let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    assert_eq!(send(&mut client, &["PING"]), "+PONG\r\n");
    assert_eq!(send(&mut client, &["SET", "greeting", "hello"]), "+OK\r\n");
    assert_eq!(send(&mut client, &["GET", "greeting"]), "$5\r\nhello\r\n");
    assert_eq!(
        send(&mut client, &["RPUSH", "colors", "red", "blue"]),
        ":2\r\n"
    );
    assert_eq!(
        send(&mut client, &["LRANGE", "colors", "0", "-1"]),
        "*2\r\n$3\r\nred\r\n$4\r\nblue\r\n"
    );
    assert_eq!(
        send(&mut client, &["HSET", "user", "name", "alice"]),
        ":1\r\n"
    );
    assert_eq!(
        send(&mut client, &["HGET", "user", "name"]),
        "$5\r\nalice\r\n"
    );
    assert_eq!(send(&mut client, &["SADD", "tags", "rust"]), ":1\r\n");
    assert_eq!(send(&mut client, &["SISMEMBER", "tags", "rust"]), ":1\r\n");
    assert_eq!(
        send(&mut client, &["SET", "temporary", "value", "EX", "60"]),
        "+OK\r\n"
    );
    assert_eq!(send(&mut client, &["PERSIST", "temporary"]), ":1\r\n");
    assert_eq!(send(&mut client, &["SET", "deleted", "value"]), "+OK\r\n");
    assert_eq!(send(&mut client, &["DEL", "deleted"]), ":1\r\n");
    drop(client);
    stop_server(server);

    assert!(std::fs::metadata(&aof).unwrap().len() > 0);

    server = start_server(port, &aof);
    let mut client = TcpStream::connect(("127.0.0.1", port)).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();

    assert_eq!(send(&mut client, &["GET", "greeting"]), "$5\r\nhello\r\n");
    assert_eq!(
        send(&mut client, &["LRANGE", "colors", "0", "-1"]),
        "*2\r\n$3\r\nred\r\n$4\r\nblue\r\n"
    );
    assert_eq!(
        send(&mut client, &["HGET", "user", "name"]),
        "$5\r\nalice\r\n"
    );
    assert_eq!(send(&mut client, &["SISMEMBER", "tags", "rust"]), ":1\r\n");
    assert_eq!(send(&mut client, &["EXISTS", "deleted"]), ":0\r\n");
    assert_eq!(send(&mut client, &["TTL", "temporary"]), ":-1\r\n");

    drop(client);
    stop_server(server);
    let _ = std::fs::remove_file(aof);
}
