use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{PersistentOrdMap, PersistentOrdMapStorage};

impl<K, V> Serialize for PersistentOrdMap<K, V>
where
    K: Clone + Ord + Serialize,
    V: Clone + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_map(self.iter())
    }
}

impl<'de, K, V> Deserialize<'de> for PersistentOrdMap<K, V>
where
    K: Clone + Ord + Deserialize<'de>,
    V: Clone + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            storage: PersistentOrdMapStorage::Exclusive(BTreeMap::deserialize(deserializer)?),
            retained_charge: None,
        })
    }
}
