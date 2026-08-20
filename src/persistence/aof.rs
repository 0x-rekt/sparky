use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Context;
use bytes::BytesMut;

use crate::{
    commands::handle_command,
    db::Db,
    resp::{RespValue, parser::parse_message, serializer},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FsyncPolicy {
    Always,
    EverySecond,
    No,
}

impl FsyncPolicy {
    pub fn from_env() -> anyhow::Result<Self> {
        match std::env::var("SPARKY_AOF_FSYNC")
            .unwrap_or_else(|_| "everysec".to_string())
            .to_ascii_lowercase()
            .as_str()
        {
            "always" => Ok(Self::Always),
            "everysec" | "every-second" => Ok(Self::EverySecond),
            "no" => Ok(Self::No),
            value => Err(anyhow::anyhow!(
                "invalid SPARKY_AOF_FSYNC '{value}'; expected always, everysec, or no"
            )),
        }
    }
}

struct AofWriter {
    writer: BufWriter<File>,
    policy: FsyncPolicy,
    last_sync: Instant,
}

#[derive(Clone)]
pub struct Aof {
    writer: Arc<Mutex<AofWriter>>,
}

impl Aof {
    pub fn open_with_policy(path: impl AsRef<Path>, policy: FsyncPolicy) -> anyhow::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_ref())
            .with_context(|| format!("failed to open AOF at {}", path.as_ref().display()))?;

        Ok(Self {
            writer: Arc::new(Mutex::new(AofWriter {
                writer: BufWriter::new(file),
                policy,
                last_sync: Instant::now(),
            })),
        })
    }

    pub fn append(&self, command: &RespValue) -> anyhow::Result<()> {
        let encoded = serializer::serialize(command);
        let mut writer = self.writer.lock().unwrap();
        writer.writer.write_all(&encoded)?;

        match writer.policy {
            FsyncPolicy::Always => sync_writer(&mut writer)?,
            FsyncPolicy::EverySecond if writer.last_sync.elapsed() >= Duration::from_secs(1) => {
                sync_writer(&mut writer)?;
            }
            FsyncPolicy::EverySecond | FsyncPolicy::No => {}
        }
        Ok(())
    }

    pub fn sync_if_due(&self) -> anyhow::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        if writer.policy == FsyncPolicy::EverySecond
            && writer.last_sync.elapsed() >= Duration::from_secs(1)
        {
            sync_writer(&mut writer)?;
        }
        Ok(())
    }

    pub fn replay(path: impl AsRef<Path>, db: &mut Db) -> anyhow::Result<()> {
        let data = std::fs::read(path.as_ref())
            .with_context(|| format!("failed to read AOF at {}", path.as_ref().display()))?;
        let mut buffer = BytesMut::from(data.as_slice());

        while !buffer.is_empty() {
            let (command, consumed) = parse_message(&buffer)
                .map_err(|error| anyhow::anyhow!("invalid AOF entry: {error}"))?;
            handle_command(command, db);
            let _ = buffer.split_to(consumed);
        }

        Ok(())
    }
}

fn sync_writer(writer: &mut AofWriter) -> anyhow::Result<()> {
    writer.writer.flush()?;
    writer.writer.get_ref().sync_data()?;
    writer.last_sync = Instant::now();
    Ok(())
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
            | "FLUSHALL"
            | "FLUSHDB"
    )
}
