use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::Context;
use bytes::BytesMut;

use crate::{
    commands::handle_command,
    db::SharedDb,
    resp::{RespValue, parser::parse_message, serializer},
};

#[derive(Clone)]
pub struct Aof {
    writer: Arc<Mutex<BufWriter<File>>>,
}

impl Aof {
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .with_context(|| format!("failed to open AOF at {}", path.as_ref().display()))?;

        Ok(Self {
            writer: Arc::new(Mutex::new(BufWriter::new(file))),
        })
    }

    pub fn append(&self, command: &RespValue) -> anyhow::Result<()> {
        let encoded = serializer::serialize(command);
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(&encoded)?;
        writer.flush()?;
        writer.get_ref().sync_data()?;
        Ok(())
    }

    pub fn replay(path: impl AsRef<Path>, db: SharedDb) -> anyhow::Result<()> {
        let data = std::fs::read(path.as_ref())
            .with_context(|| format!("failed to read AOF at {}", path.as_ref().display()))?;
        let mut buffer = BytesMut::from(data.as_slice());

        while !buffer.is_empty() {
            let (command, consumed) = parse_message(&buffer)
                .map_err(|error| anyhow::anyhow!("invalid AOF entry: {error}"))?;
            handle_command(command, db.clone());
            let _ = buffer.split_to(consumed);
        }

        Ok(())
    }
}

pub fn is_write_command(request: &RespValue) -> bool {
    let RespValue::Array(parts) = request else {
        return false;
    };
    let Some(RespValue::BulkString(command)) = parts.first() else {
        return false;
    };

    matches!(
        String::from_utf8_lossy(command)
            .to_ascii_uppercase()
            .as_str(),
        "SET"
            | "DEL"
            | "MSET"
            | "INCR"
            | "DECR"
            | "INCRBY"
            | "APPEND"
            | "GETSET"
            | "GETDEL"
            | "EXPIRE"
            | "PEXPIRE"
            | "RPUSH"
            | "LPUSH"
            | "LPOP"
            | "RPOP"
            | "LSET"
            | "LREM"
            | "HSET"
            | "HDEL"
            | "HINCRBY"
            | "SADD"
            | "SREM"
            | "RENAME"
            | "PERSIST"
    )
}
