use std::time::{Duration, Instant};

use super::SharedDb;

pub fn clear_expired_keys(db: &SharedDb) {
    let now = Instant::now();
    let mut database = db.lock().unwrap();
    let expired_keys: Vec<String> = database
        .expirations
        .iter()
        .filter_map(|(key, &expiration)| (expiration <= now).then_some(key.clone()))
        .collect();

    for key in expired_keys {
        database.remove(&key);
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
