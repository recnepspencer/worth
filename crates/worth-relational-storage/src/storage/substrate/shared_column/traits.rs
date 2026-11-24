//! Plain owner views preserve the original value wire, never carrier internals.
use super::{SharedColumn, SharedColumnIter, StoredValue};
impl<T: StoredValue> Default for SharedColumn<T> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}
impl<T: StoredValue> std::ops::Index<usize> for SharedColumn<T> {
    type Output = T::Value;
    fn index(&self, index: usize) -> &T::Value {
        self.get(index).expect("selected column index is in range")
    }
}
impl<T: StoredValue> std::ops::IndexMut<usize> for SharedColumn<T> {
    fn index_mut(&mut self, index: usize) -> &mut T::Value {
        self.get_mut(index)
            .expect("selected column index is in range")
    }
}
impl<T: StoredValue> From<Vec<T::Value>> for SharedColumn<T> {
    fn from(values: Vec<T::Value>) -> Self {
        Self::from_dense_values(values)
    }
}
impl<T: StoredValue> FromIterator<T::Value> for SharedColumn<T> {
    fn from_iter<I: IntoIterator<Item = T::Value>>(values: I) -> Self {
        let mut column = Self::default();
        for value in values {
            column.push(value);
        }
        column
    }
}
impl<'a, T: StoredValue> IntoIterator for &'a SharedColumn<T> {
    type Item = &'a T::Value;
    type IntoIter = SharedColumnIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T: StoredValue> IntoIterator for SharedColumn<T> {
    type Item = T::Value;
    type IntoIter = std::vec::IntoIter<T::Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
impl<T: StoredValue> serde::Serialize for SharedColumn<T>
where
    T::Value: serde::Serialize,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
