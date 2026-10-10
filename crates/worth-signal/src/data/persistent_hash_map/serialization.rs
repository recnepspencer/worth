use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::PersistentHashMap;

#[cfg(test)]
#[path = "serialization/tests.rs"]
mod tests;

impl<K, V> Serialize for PersistentHashMap<K, V>
where
    K: Clone + Eq + Hash + Ord + Serialize,
    V: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Reconstructive serialization: O(n log n) key comparisons and O(n)
        // borrowed-entry scratch. Ordinary lookup and mutation do not sort.
        let mut entries: Vec<_> = self.iter().collect();
        entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        serializer.collect_map(entries)
    }
}

impl<'de, K, V> Deserialize<'de> for PersistentHashMap<K, V>
where
    K: Clone + Eq + Hash + Deserialize<'de>,
    V: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(HashMap::<K, V>::deserialize(deserializer)?
            .into_iter()
            .collect())
    }
}
