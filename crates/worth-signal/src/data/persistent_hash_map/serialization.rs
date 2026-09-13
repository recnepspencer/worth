use std::collections::HashMap;
use std::hash::Hash;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::PersistentHashMap;

impl<K, V> Serialize for PersistentHashMap<K, V>
where
    K: Clone + Eq + Hash + Serialize,
    V: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_map(self.iter())
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
