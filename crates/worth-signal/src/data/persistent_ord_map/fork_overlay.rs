use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use super::entry_handle::{self, SharedKey};

pub(super) fn base_value_if_live<'a, K, V, Q>(
    base: &'a BTreeMap<K, V>,
    retired_intervals: &im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &Q,
) -> Option<&'a V>
where
    K: Clone + Ord + std::borrow::Borrow<Q>,
    Q: Ord + ?Sized,
{
    let (base_key, value) = base.get_key_value(key)?;
    retired_interval_end(retired_intervals, base_key)
        .is_none()
        .then_some(value)
}

pub(super) fn retired_interval_end<'a, K: Clone + Ord>(
    retired_intervals: &'a im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &K,
) -> Option<&'a SharedKey<K>> {
    entry_handle::get(retired_intervals, key).or_else(|| {
        entry_handle::previous_after_exact_miss(retired_intervals, key)
            .and_then(|(_, end)| (end.as_key() >= key).then_some(end))
    })
}

pub(super) fn record_base_retirement<K: Clone + Ord, V>(
    base: &BTreeMap<K, V>,
    retired_intervals: &mut im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &K,
) {
    let left_start = base
        .range(..key)
        .next_back()
        .and_then(|(predecessor, _)| containing_interval(retired_intervals, predecessor))
        .map(|(start, _)| start);
    let right_start = base
        .range((Excluded(key), Unbounded))
        .next()
        .and_then(|(successor, _)| {
            entry_handle::get_key_value(retired_intervals, successor)
                .map(|(start, _)| start.clone())
        });
    let start = left_start
        .clone()
        .unwrap_or_else(|| SharedKey::new(key.clone()));
    let end = right_start
        .as_ref()
        .and_then(|right| entry_handle::get(retired_intervals, right.as_key()))
        .cloned()
        .unwrap_or_else(|| SharedKey::new(key.clone()));

    if let Some(left_start) = left_start {
        entry_handle::remove(retired_intervals, left_start.as_key());
    }
    if let Some(right_start) = right_start {
        entry_handle::remove(retired_intervals, right_start.as_key());
    }
    retired_intervals.insert(start, end);
}

pub(super) fn record_base_readmission<K: Clone + Ord, V>(
    base: &BTreeMap<K, V>,
    retired_intervals: &mut im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &K,
) {
    let Some((start, end)) = containing_interval(retired_intervals, key) else {
        return;
    };
    entry_handle::remove(retired_intervals, start.as_key());
    if start.as_key() < key {
        let predecessor = base
            .range(..key)
            .next_back()
            .map(|(predecessor, _)| SharedKey::new(predecessor.clone()))
            .expect("non-start interval key must have a base predecessor");
        retired_intervals.insert(start, predecessor);
    }
    if key < end.as_key() {
        let successor = base
            .range((Excluded(key), Unbounded))
            .next()
            .map(|(successor, _)| SharedKey::new(successor.clone()))
            .expect("non-end interval key must have a base successor");
        retired_intervals.insert(successor, end);
    }
}

fn containing_interval<K: Clone + Ord>(
    retired_intervals: &im::OrdMap<SharedKey<K>, SharedKey<K>>,
    key: &K,
) -> Option<(SharedKey<K>, SharedKey<K>)> {
    entry_handle::get_key_value(retired_intervals, key)
        .map(|(start, end)| (start.clone(), end.clone()))
        .or_else(|| {
            entry_handle::previous_after_exact_miss(retired_intervals, key)
                .filter(|(_, end)| end.as_key() >= key)
                .map(|(start, end)| (start.clone(), end.clone()))
        })
}
