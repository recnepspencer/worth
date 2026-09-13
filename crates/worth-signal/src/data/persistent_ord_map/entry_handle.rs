use std::borrow::Borrow;
use std::cmp::Ordering;
use std::sync::Arc;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SharedKey<K>(Arc<K>);

impl<K> SharedKey<K> {
    pub(super) fn new(key: K) -> Self {
        Self(Arc::new(key))
    }

    pub(super) fn as_key(&self) -> &K {
        self.0.as_ref()
    }
}

impl<K> Clone for SharedKey<K> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<K> Borrow<K> for SharedKey<K> {
    fn borrow(&self) -> &K {
        self.as_key()
    }
}

trait KeyOrder<Q: Ord + ?Sized> {
    fn borrowed(&self) -> &Q;
}

impl<K, Q> KeyOrder<Q> for SharedKey<K>
where
    K: Borrow<Q>,
    Q: Ord + ?Sized,
{
    fn borrowed(&self) -> &Q {
        self.as_key().borrow()
    }
}

struct KeyQuery<'a, Q: ?Sized>(&'a Q);

impl<Q: Ord + ?Sized> KeyOrder<Q> for KeyQuery<'_, Q> {
    fn borrowed(&self) -> &Q {
        self.0
    }
}

impl<Q: Ord + ?Sized> PartialEq for dyn KeyOrder<Q> + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.borrowed() == other.borrowed()
    }
}

impl<Q: Ord + ?Sized> Eq for dyn KeyOrder<Q> + '_ {}

impl<Q: Ord + ?Sized> PartialOrd for dyn KeyOrder<Q> + '_ {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Q: Ord + ?Sized> Ord for dyn KeyOrder<Q> + '_ {
    fn cmp(&self, other: &Self) -> Ordering {
        self.borrowed().cmp(other.borrowed())
    }
}

impl<'view, K, Q> Borrow<dyn KeyOrder<Q> + 'view> for SharedKey<K>
where
    K: Borrow<Q> + 'view,
    Q: Ord + ?Sized + 'view,
{
    fn borrow(&self) -> &(dyn KeyOrder<Q> + 'view) {
        self
    }
}

pub(super) fn get<'map, K, Q, V>(map: &'map im::OrdMap<SharedKey<K>, V>, key: &Q) -> Option<&'map V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    let query = KeyQuery(key);
    map.get(&query as &dyn KeyOrder<Q>)
}

pub(super) fn get_key_value<'map, K, Q, V>(
    map: &'map im::OrdMap<SharedKey<K>, V>,
    key: &Q,
) -> Option<(&'map SharedKey<K>, &'map V)>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    let query = KeyQuery(key);
    map.get_key_value(&query as &dyn KeyOrder<Q>)
}

pub(super) fn get_mut<'map, K, Q, V>(
    map: &'map mut im::OrdMap<SharedKey<K>, V>,
    key: &Q,
) -> Option<&'map mut V>
where
    K: Clone + Ord + Borrow<Q>,
    Q: Ord + ?Sized,
    V: Clone,
{
    let query = KeyQuery(key);
    map.get_mut(&query as &dyn KeyOrder<Q>)
}

pub(super) fn remove<K, Q, V>(map: &mut im::OrdMap<SharedKey<K>, V>, key: &Q) -> Option<V>
where
    K: Clone + Ord + Borrow<Q>,
    Q: Ord + ?Sized,
    V: Clone,
{
    let query = KeyQuery(key);
    map.remove(&query as &dyn KeyOrder<Q>)
}

pub(super) fn previous_after_exact_miss<'map, K, Q, V>(
    map: &'map im::OrdMap<SharedKey<K>, V>,
    key: &Q,
) -> Option<(&'map SharedKey<K>, &'map V)>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    let query = KeyQuery(key);
    map.get_prev(&query as &dyn KeyOrder<Q>)
}
