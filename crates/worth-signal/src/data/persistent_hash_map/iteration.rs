use std::hash::Hash;
use std::sync::Arc;

use super::entry_handle::SharedKey;

pub(crate) enum PersistentHashMapIter<'a, K, V> {
    Exclusive(std::collections::hash_map::Iter<'a, K, V>),
    ForkShared {
        base: std::collections::hash_map::Iter<'a, K, V>,
        changes: im::hashmap::Iter<'a, SharedKey<K>, Option<Arc<V>>>,
        changed_keys: &'a im::HashMap<SharedKey<K>, Option<Arc<V>>>,
        remaining: usize,
    },
}

impl<'a, K, V> Iterator for PersistentHashMapIter<'a, K, V>
where
    K: Eq + Hash,
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Exclusive(iter) => iter.next(),
            Self::ForkShared {
                base,
                changes,
                changed_keys,
                remaining,
            } => {
                for (key, value) in base.by_ref() {
                    if !changed_keys.contains_key(key) {
                        *remaining -= 1;
                        return Some((key, value));
                    }
                }
                let next = changes.find_map(|(key, value)| {
                    value.as_ref().map(|value| (key.as_key(), value.as_ref()))
                });
                if next.is_some() {
                    *remaining -= 1;
                }
                next
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Exclusive(iter) => iter.size_hint(),
            Self::ForkShared { remaining, .. } => (*remaining, Some(*remaining)),
        }
    }
}
