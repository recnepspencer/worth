#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderedStreamMergeError<K> {
    DuplicateKey(K),
}

pub trait OrderedStreamItem {
    type OrderKey: Ord + Copy;

    fn order_key(&self) -> Self::OrderKey;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocallyOrderedShard<T> {
    items: Vec<T>,
}

impl<T: OrderedStreamItem> LocallyOrderedShard<T> {
    pub fn new(items: impl IntoIterator<Item = T>) -> Self {
        let items = items.into_iter().collect::<Vec<_>>();
        assert_strict_order(items.as_slice());
        Self { items }
    }

    pub fn canonicalize_unordered(items: impl IntoIterator<Item = T>) -> Self {
        let mut items = items.into_iter().collect::<Vec<_>>();
        if items.len() > 1 {
            items.sort_unstable_by_key(|item| item.order_key());
        }
        Self::new(items)
    }

    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MergeableOrderedStream<T> {
    shards: Vec<LocallyOrderedShard<T>>,
}

impl<T> MergeableOrderedStream<T> {
    pub fn new(shards: impl IntoIterator<Item = LocallyOrderedShard<T>>) -> Self {
        Self {
            shards: shards.into_iter().collect(),
        }
    }
}

impl<T: OrderedStreamItem> MergeableOrderedStream<T> {
    pub fn try_into_vec(self) -> Result<Vec<T>, OrderedStreamMergeError<T::OrderKey>> {
        let mut merged = Vec::<T>::new();
        for shard in self.shards {
            merged = merge_ordered_streams(merged, shard.into_vec())?;
        }
        Ok(merged)
    }
}

fn assert_strict_order<T: OrderedStreamItem>(items: &[T]) {
    for pair in items.windows(2) {
        if let [left, right] = pair {
            assert!(
                left.order_key() < right.order_key(),
                "ordered shard must be strictly increasing by stream key"
            );
        }
    }
}

fn merge_ordered_streams<T: OrderedStreamItem>(
    left: Vec<T>,
    right: Vec<T>,
) -> Result<Vec<T>, OrderedStreamMergeError<T::OrderKey>> {
    let mut merged = Vec::with_capacity(left.len() + right.len());
    let mut left = left.into_iter().peekable();
    let mut right = right.into_iter().peekable();

    loop {
        match (left.peek(), right.peek()) {
            (Some(left_item), Some(right_item)) => {
                match left_item.order_key().cmp(&right_item.order_key()) {
                    std::cmp::Ordering::Less => {
                        merged.push(left.next().expect("left item should exist"));
                    }
                    std::cmp::Ordering::Greater => {
                        merged.push(right.next().expect("right item should exist"));
                    }
                    std::cmp::Ordering::Equal => {
                        return Err(OrderedStreamMergeError::DuplicateKey(left_item.order_key()));
                    }
                }
            }
            (Some(_), None) => {
                merged.extend(left);
                return Ok(merged);
            }
            (None, Some(_)) => {
                merged.extend(right);
                return Ok(merged);
            }
            (None, None) => return Ok(merged),
        }
    }
}
