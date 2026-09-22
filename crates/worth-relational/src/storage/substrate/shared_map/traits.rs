use super::{MapNode, SharedMap, SharedMapIter};

impl<K: Ord + Copy, V: Clone> Default for SharedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Copy, V: Clone> FromIterator<(K, V)> for SharedMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(values: I) -> Self {
        let mut map = Self::new();
        for (key, value) in values {
            map.insert(key, value);
        }
        map
    }
}

impl<'a, K: Ord + Copy, V: Clone> IntoIterator for &'a SharedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = SharedMapIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K: Ord + Copy, V: Clone> IntoIterator for SharedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut values = Vec::with_capacity(self.len());
        MapNode::into_values(self.root, &mut values);
        values.into_iter()
    }
}

impl<K: Ord + Copy, V: Clone> std::ops::Index<&K> for SharedMap<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("map key is absent")
    }
}

impl<K: Ord + Copy, V: Clone + PartialEq> PartialEq for SharedMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<K: Ord + Copy, V: Clone + Eq> Eq for SharedMap<K, V> {}
