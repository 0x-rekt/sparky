use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}, time::{Duration, Instant}};

use bytes::Bytes;

pub struct Db {
    pub strings: HashMap<String, Bytes>,
    pub lists: HashMap<String, Vec<Bytes>>,
    pub hashes: HashMap<String, HashMap<String, Bytes>>,
    pub sets: HashMap<String, HashSet<Bytes>>,
    pub expirations: HashMap<String, Instant>,
}

impl Db {
    pub fn new() -> Self {
        Self {
            strings: HashMap::new(),
            lists: HashMap::new(),
            hashes: HashMap::new(),
            sets: HashMap::new(),
            expirations: HashMap::new(),
        }
    }

    pub fn insert_with_expiry(&mut self, key: String, value: Bytes, expiry: Option<Instant>) {
        self.strings.insert(key.clone(), value);
        match expiry {
            Some(exp) => {
                self.expirations.insert(key, exp);
            }
            None => {
                self.expirations.remove(&key);
            }
        }
    }

    pub fn get(&self, key: &String) -> Option<&Bytes> {
        let now = Instant::now();
        if let Some(expiration) = self.expirations.get(key) {
            if *expiration <= now {
                return None;
            }
        }
        self.strings.get(key)
    }

    pub fn remove(&mut self, key: &String) -> Option<Bytes> {
        self.lists.remove(key);
        self.hashes.remove(key);
        self.sets.remove(key);
        self.expirations.remove(key);
        self.strings.remove(key)
    }
}

pub type SharedDb = std::sync::Arc<std::sync::Mutex<Db>>;

pub fn create_shared_db() -> SharedDb {
    Arc::new(Mutex::new(Db::new()))
}

pub fn clear_expired_keys(db: &SharedDb) {
    let now = Instant::now();
    let mut db_lock = db.lock().unwrap();
    let expired_keys: Vec<String> = db_lock
        .expirations
        .iter()
        .filter_map(|(key, &expiration)| {
            if expiration <= now {
                Some(key.clone())
            } else {
                None
            }
        })
        .collect();

    for key in expired_keys {
        db_lock.remove(&key);
    }
}

pub fn spawn_expiry_cleaner(db: SharedDb) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            clear_expired_keys(&db);
        }
    });
}