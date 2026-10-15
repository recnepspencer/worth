#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmpty<T> {
    items: Vec<T>,
}

impl<T> NonEmpty<T> {
    pub fn new(head: T, tail: Vec<T>) -> Self {
        let mut items = Vec::with_capacity(1 + tail.len());
        items.push(head);
        items.extend(tail);
        Self { items }
    }

    pub fn try_from_vec(items: Vec<T>) -> Result<Self, Vec<T>> {
        if items.is_empty() {
            Err(items)
        } else {
            Ok(Self { items })
        }
    }

    pub fn first(&self) -> &T {
        &self.items[0]
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub const fn is_empty(&self) -> bool {
        false
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    pub fn map<U>(self, mut transform: impl FnMut(T) -> U) -> NonEmpty<U> {
        NonEmpty {
            items: self.items.into_iter().map(&mut transform).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::NonEmpty;

    #[test]
    fn non_empty_builds_from_head_and_tail() {
        let items = NonEmpty::new(1, vec![2, 3]);

        assert_eq!(items.first(), &1);
        assert_eq!(items.len(), 3);
        assert_eq!(items.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn non_empty_rejects_empty_vec() {
        assert_eq!(NonEmpty::<u8>::try_from_vec(Vec::new()), Err(Vec::new()));
    }

    #[test]
    fn map_preserves_nonempty_cardinality_and_order() {
        let mapped = NonEmpty::new(1, vec![2, 3]).map(|value| value * 10);

        assert_eq!(mapped.as_slice(), &[10, 20, 30]);
    }

    const _: () = assert!(size_of::<NonEmpty<u64>>() == size_of::<Vec<u64>>());
}
