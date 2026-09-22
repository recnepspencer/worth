use super::{SharedColumn, SharedColumnIter};

impl<T: Clone> Default for SharedColumn<T> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<T: Clone> std::ops::Index<usize> for SharedColumn<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        self.get(index).expect("column index out of bounds")
    }
}

impl<T: Clone> std::ops::IndexMut<usize> for SharedColumn<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).expect("column index out of bounds")
    }
}

impl<T: Clone> From<Vec<T>> for SharedColumn<T> {
    fn from(values: Vec<T>) -> Self {
        values.into_iter().collect()
    }
}

impl<T: Clone> FromIterator<T> for SharedColumn<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut column = Self::default();
        for value in values {
            column.push(value);
        }
        column
    }
}

impl<'a, T: Clone> IntoIterator for &'a SharedColumn<T> {
    type Item = &'a T;
    type IntoIter = SharedColumnIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone> IntoIterator for SharedColumn<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}

impl<T: Clone + serde::Serialize> serde::Serialize for SharedColumn<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
