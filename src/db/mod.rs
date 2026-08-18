use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}, time::Instant};

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

    pub fn insert(&mut self, key: String, value: Bytes) {
        self.strings.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<&Bytes> {
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