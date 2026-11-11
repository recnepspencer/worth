use super::{MapNode, SharedMap, SharedMapIter};

impl<K: Ord + Copy, V: super::StoredValue> Default for SharedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Copy, V: super::StoredValue> FromIterator<(K, V::Value)> for SharedMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V::Value)>>(values: I) -> Self {
        let mut map = Self::new();
        for (key, value) in values {
            map.insert(key, value);
        }
        map
    }
}

impl<'a, K: Ord + Copy, V: super::StoredValue> IntoIterator for &'a SharedMap<K, V> {
    type Item = (&'a K, &'a V::Value);
    type IntoIter = SharedMapIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K: Ord + Copy, V: super::StoredValue> IntoIterator for SharedMap<K, V> {
    type Item = (K, V::Value);
    type IntoIter = std::vec::IntoIter<(K, V::Value)>;
    fn into_iter(self) -> Self::IntoIter {
        let mut values = Vec::with_capacity(self.len());
        MapNode::into_values(self.root, &mut values);
        values.into_iter()
    }
}

impl<K: Ord + Copy, V: super::StoredValue> std::ops::Index<&K> for SharedMap<K, V> {
    type Output = V::Value;
    fn index(&self, key: &K) -> &V::Value {
        self.get(key).expect(/* Proof: Private map indexing callers first select the key from the same immutable owner map; external lookup uses the Option-returning get door. */ "map key is absent")
    }
}

impl<K: Ord + Copy, V: super::StoredValue> PartialEq for SharedMap<K, V>
where
    V::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}
impl<K: Ord + Copy, V: super::StoredValue> Eq for SharedMap<K, V> where V::Value: Eq {}
