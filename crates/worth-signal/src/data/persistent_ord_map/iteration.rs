use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use super::entry_handle::SharedKey;
use super::fork_overlay::retired_interval_end;

pub(crate) struct LiveBaseIter<'a, K: Clone + Ord, V> {
    base: &'a BTreeMap<K, V>,
    range: std::collections::btree_map::Range<'a, K, V>,
    retired_intervals: &'a im::OrdMap<SharedKey<K>, SharedKey<K>>,
}

impl<'a, K: Clone + Ord, V> LiveBaseIter<'a, K, V> {
    pub(super) fn new(
        base: &'a BTreeMap<K, V>,
        retired_intervals: &'a im::OrdMap<SharedKey<K>, SharedKey<K>>,
    ) -> Self {
        Self {
            base,
            range: base.range(..),
            retired_intervals,
        }
    }
}

impl<'a, K: Clone + Ord, V> Iterator for LiveBaseIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (key, value) = self.range.next()?;
            let Some(retired_end) = retired_interval_end(self.retired_intervals, key) else {
                return Some((key, value));
            };
            self.range = self.base.range((Excluded(retired_end.as_key()), Unbounded));
        }
    }
}

pub(crate) enum PersistentOrdMapIter<'a, K: Clone + Ord, V: Clone> {
    Exclusive(std::collections::btree_map::Iter<'a, K, V>),
    ForkShared {
        base: std::iter::Peekable<LiveBaseIter<'a, K, V>>,
        changes: std::iter::Peekable<im::ordmap::Iter<'a, SharedKey<K>, std::sync::Arc<V>>>,
        remaining: usize,
    },
}

impl<'a, K: Clone + Ord, V: Clone> Iterator for PersistentOrdMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let Self::ForkShared {
            base,
            changes,
            remaining,
        } = self
        else {
            return match self {
                Self::Exclusive(values) => values.next(),
                Self::ForkShared { .. } => unreachable!(),
            };
        };
        let next = match (base.peek(), changes.peek()) {
            (Some((base_key, _)), Some((change_key, _))) if *base_key < change_key.as_key() => {
                base.next()
            }
            (Some((base_key, _)), Some((change_key, _))) if *base_key == change_key.as_key() => {
                base.next();
                changes
                    .next()
                    .map(|(key, value)| (key.as_key(), value.as_ref()))
            }
            (_, Some(_)) => changes
                .next()
                .map(|(key, value)| (key.as_key(), value.as_ref())),
            (Some(_), None) => base.next(),
            (None, None) => None,
        };
        if next.is_some() {
            *remaining -= 1;
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Exclusive(values) => values.size_hint(),
            Self::ForkShared { remaining, .. } => (*remaining, Some(*remaining)),
        }
    }
}
