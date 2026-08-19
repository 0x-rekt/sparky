use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    time::Instant,
};

use bytes::Bytes;

mod expiry;
pub use expiry::spawn_expiry_cleaner;

pub struct Db {
    pub strings: HashMap<String, Bytes>,
    pub lists: HashMap<String, VecDeque<Bytes>>,
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
        if let Some(expiration) = self.expirations.get(key)
            && *expiration <= now
        {
            return None;
        }
        self.strings.get(key)
    }

    pub fn contains_key(&mut self, key: &String) -> bool {
        self.remove_if_expired(key);
        self.strings.contains_key(key)
            || self.lists.contains_key(key)
            || self.hashes.contains_key(key)
            || self.sets.contains_key(key)
    }

    pub fn keys(&mut self) -> impl Iterator<Item = String> {
        let now = Instant::now();
        let expired_keys: Vec<String> = self
            .expirations
            .iter()
            .filter_map(|(key, &expiration)| (expiration <= now).then_some(key.clone()))
            .collect();
        for key in expired_keys {
            self.remove(&key);
        }

        let mut keys = HashSet::new();
        keys.extend(self.strings.keys().cloned());
        keys.extend(self.lists.keys().cloned());
        keys.extend(self.hashes.keys().cloned());
        keys.extend(self.sets.keys().cloned());
        keys.into_iter()
    }

    pub fn set_expiry(&mut self, key: &String, expiry: Instant) -> bool {
        if !self.contains_key(key) {
            return false;
        }

        self.expirations.insert(key.clone(), expiry);
        if expiry <= Instant::now() {
            self.remove(key);
        }
        true
    }

    pub fn ttl(&mut self, key: &String, in_millis: bool) -> i64 {
        if !self.contains_key(key) {
            return -2;
        }

        let Some(expiry) = self.expirations.get(key).copied() else {
            return -1;
        };

        let remaining = expiry.saturating_duration_since(Instant::now());
        if in_millis {
            remaining.as_millis().min(i64::MAX as u128) as i64
        } else {
            remaining.as_secs().min(i64::MAX as u64) as i64
        }
    }

    fn remove_if_expired(&mut self, key: &String) {
        if self
            .expirations
            .get(key)
            .is_some_and(|expiration| *expiration <= Instant::now())
        {
            self.remove(key);
        }
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
