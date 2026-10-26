//! Allocation-free predecessor search carrying separators across tree levels.
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;

use super::entry_handle::{self, SharedKey};

trait SeekKey<K, Q: Ord + ?Sized> {
    fn borrowed(&self) -> &Q;
    fn shared(&self) -> Option<&SharedKey<K>> {
        None
    }
    fn observe(&self, _key: &SharedKey<K>, _ordering: Ordering) {}
}

impl<K: Borrow<Q>, Q: Ord + ?Sized> SeekKey<K, Q> for SharedKey<K> {
    fn borrowed(&self) -> &Q {
        self.as_key().borrow()
    }
    fn shared(&self) -> Option<&SharedKey<K>> {
        Some(self)
    }
}

struct Query<'key, K, Q: ?Sized> {
    key: &'key Q,
    lower: RefCell<Option<SharedKey<K>>>,
}

impl<K: Borrow<Q>, Q: Ord + ?Sized> SeekKey<K, Q> for Query<'_, K, Q> {
    fn borrowed(&self) -> &Q {
        self.key
    }
    fn observe(&self, key: &SharedKey<K>, ordering: Ordering) {
        if ordering == Ordering::Less {
            let mut lower = self.lower.borrow_mut();
            if lower
                .as_ref()
                .is_none_or(|previous| previous.as_key().borrow() < key.as_key().borrow())
            {
                *lower = Some(key.clone());
            }
        }
    }
}

impl<K, Q: Ord + ?Sized> PartialEq for dyn SeekKey<K, Q> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}
impl<K, Q: Ord + ?Sized> Eq for dyn SeekKey<K, Q> + '_ {}
impl<K, Q: Ord + ?Sized> PartialOrd for dyn SeekKey<K, Q> + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<K, Q: Ord + ?Sized> Ord for dyn SeekKey<K, Q> + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        let ordering = self.borrowed().cmp(other.borrowed());
        if let Some(key) = self.shared() {
            other.observe(key, ordering);
        }
        if let Some(key) = other.shared() {
            self.observe(key, ordering.reverse());
        }
        ordering
    }
}

impl<'view, K: Borrow<Q> + 'view, Q: Ord + ?Sized + 'view> Borrow<dyn SeekKey<K, Q> + 'view>
    for SharedKey<K>
{
    fn borrow(&self) -> &(dyn SeekKey<K, Q> + 'view) {
        self
    }
}

/// O(log n) comparisons and Arc-handle copies, O(1) stack, no allocation or
/// key-payload clone. im 15.1's get_prev can lose a parent separator when its
/// child has no predecessor. Remember the greatest lower key encountered by
/// ordinary search, then resolve its handle back to the map's borrowed entry.
pub(super) fn get<'map, K: Ord + Borrow<Q>, Q: Ord + ?Sized, V>(
    map: &'map im::OrdMap<SharedKey<K>, V>,
    key: &Q,
) -> Option<(&'map SharedKey<K>, &'map V)> {
    let query: Query<'_, K, Q> = Query {
        key,
        lower: RefCell::new(None),
    };
    if let Some(exact) = map.get_key_value(&query as &dyn SeekKey<K, Q>) {
        return Some(exact);
    }
    let lower = query.lower.into_inner()?;
    entry_handle::get_key_value::<K, K, V>(map, lower.as_key())
}
