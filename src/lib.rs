use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

// Define a trait that your repositories or data sources should implement
pub trait EntityFetcher<K, V> {
    fn fetch_entity(&self, key: &K) -> Option<V>;
}

pub struct TTRCache<K, V, F>
where
    K: Eq + Hash,
    F: EntityFetcher<K, V>,
{
    ttl: Duration,
    cache: HashMap<K, (Instant, V)>,
    fetcher: F,
}

impl<K, V, F> TTRCache<K, V, F>
where
    K: Eq + Hash + Clone,
    F: EntityFetcher<K, V>,
{
    pub fn new(ttl: Duration, fetcher: F) -> Self {
        TTRCache {
            ttl,
            cache: HashMap::new(),
            fetcher,
        }
    }

    fn fetch_entity(&mut self, key: &K) {
        if let Some(entity) = self.fetcher.fetch_entity(key) {
            self.cache.insert(key.clone(), (Instant::now(), entity));
        }
    }

    fn refresh(&mut self, key: &K) {
        if !self.cache.contains_key(key) {
            self.fetch_entity(key);
            return;
        }

        if let Some((timestamp, _)) = self.cache.get(key) {
            if Instant::now().duration_since(*timestamp) >= self.ttl {
                self.fetch_entity(key);
            }
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.refresh(key);
        self.cache.get(key).map(|(_, entity)| entity)
    }
}
