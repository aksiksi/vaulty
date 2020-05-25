use std::collections::HashMap;

use chrono::prelude::*;

use vaulty::email::Email;

pub struct Cache {
    cache: HashMap<String, CacheEntry>,

    /// Total number of entries processed
    num_processed: u64,

    /// Average processing time for cache entries, in microseconds
    avg_processing_time: f32,
}

// Cache entry is cloneable to reduce read lock hold time
#[derive(Clone)]
pub struct CacheEntry {
    pub email: Email,
    pub address: vaulty::db::Address,

    // Stores the indices of successfully processed attachments
    // for this email
    pub attachments_processed: Vec<u16>,

    pub insertion_time: Option<DateTime<Local>>,
    pub last_updated: Option<DateTime<Local>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            num_processed: 0,
            avg_processing_time: 0.0,
        }
    }

    pub fn insert(&mut self, key: String, mut entry: CacheEntry) {
        entry.insertion_time = Some(Local::now());
        self.cache.insert(key, entry);
    }

    pub fn get(&self, key: &str) -> Option<&CacheEntry> {
        self.cache.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut CacheEntry> {
        let entry = self.cache.get_mut(key);

        // Update the last updated time for this entry
        entry.map(|e| {
            e.last_updated = Some(Local::now());
            e
        })
    }

    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) {
        assert!(self.contains(key));

        self.num_processed += 1;

        let entry = self.get_mut(key).unwrap();

        // Get the total number of microseconds this entry spent in the cache
        let processing_time = Local::now()
            .signed_duration_since(entry.insertion_time.unwrap())
            .num_microseconds()
            .unwrap();

        // Update the overall average processing time for the cache
        // Note that this an approximation
        self.avg_processing_time += processing_time as f32 / self.num_processed as f32;

        self.cache.remove(key);
    }
}
