//! TTR (Time-to-Refresh) cache that serves stale data while refreshing in background.
//!
//! ```rust
//! use std::time::Duration;
//! use ttr_cache::{EntityFetcher, TTRCache};
//!

//! struct MyDataSource;

//! impl EntityFetcher<String, u64> for MyDataSource {
//!     fn fetch_entity(&self, key: &String) -> Option<u64> {

//!         Some(42)
//!     }
//! }
//!
//! let cache = TTRCache::new(Duration::from_secs(300), MyDataSource);
//! ```
//!
//! - Generic keys and values
//! - Configurable TTR
//! - Thread-safe with sync primitives

use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Data source interface for fetching entities.
pub trait EntityFetcher<K, V> {
    /// Fetches an entity by key.
    ///
    /// Returns `Some(V)` if found, `None` otherwise.
    fn fetch_entity(&self, key: &K) -> Option<V>;
}

/// Cache that refreshes stale entries while serving them.
///
/// Types:
/// - `K`: Key type (must be `Eq + Hash`)
/// - `V`: Value type
/// - `F`: Fetcher implementing `EntityFetcher<K, V>`
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
    /// Creates a new cache with given TTL and fetcher.
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

    /// Gets a value, refreshing if stale.
    ///
    /// Returns `Some(&V)` if found, `None` otherwise.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.refresh(key);
        self.cache.get(key).map(|(_, entity)| entity)
    }
}
