use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

mod checked;
pub(crate) use checked::{canonical_merge_streams_checked, canonical_merge_values_checked};

#[derive(Debug)]
pub(crate) struct OrderedReductionStream<K, T> {
    items: Vec<(K, T)>,
}

impl<K, T> OrderedReductionStream<K, T>
where
    K: Ord,
{
    pub(crate) fn new(items: Vec<(K, T)>) -> Self {
        debug_assert!(
            items.windows(2).all(|window| window[0].0 <= window[1].0),
            "ordered reduction streams must be locally sorted"
        );
        Self { items }
    }

    pub(crate) fn singleton(key: K, item: T) -> Self {
        Self {
            items: vec![(key, item)],
        }
    }

    pub(crate) fn owned_allocation_capacity_bytes(
        &self,
        item_bytes: impl Fn(&K, &T) -> u64,
    ) -> u64 {
        (self.items.capacity() as u64)
            .saturating_mul(std::mem::size_of::<(K, T)>() as u64)
            .saturating_add(self.items.iter().fold(0_u64, |bytes, (key, item)| {
                bytes.saturating_add(item_bytes(key, item))
            }))
    }

    fn into_items(self) -> Vec<(K, T)> {
        self.items
    }
}

#[derive(Debug)]
struct MergeHead<K> {
    key: Reverse<K>,
    stream_index: usize,
}

impl<K: Ord> PartialEq for MergeHead<K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.stream_index == other.stream_index
    }
}

impl<K: Ord> Eq for MergeHead<K> {}

impl<K: Ord> PartialOrd for MergeHead<K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: Ord> Ord for MergeHead<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key
            .cmp(&other.key)
            .then_with(|| other.stream_index.cmp(&self.stream_index))
    }
}

pub(crate) fn canonical_merge_streams<K, T>(
    streams: Vec<OrderedReductionStream<K, T>>,
) -> Vec<(K, T)>
where
    K: Ord + Clone,
{
    let mut heap = BinaryHeap::new();
    let mut iterators = streams
        .into_iter()
        .map(|stream| stream.into_items().into_iter())
        .collect::<Vec<_>>();
    let mut current = Vec::with_capacity(iterators.len());

    for (stream_index, iterator) in iterators.iter_mut().enumerate() {
        let next = iterator.next();
        if let Some((key, item)) = next {
            heap.push(MergeHead {
                key: Reverse(key.clone()),
                stream_index,
            });
            current.push(Some((key, item)));
        } else {
            current.push(None);
        }
    }

    let mut merged = Vec::new();
    while let Some(head) = heap.pop() {
        let stream_index = head.stream_index;
        let (key, item) = current[stream_index]
            .take()
            .expect("merge head must reference a current stream item");
        merged.push((key, item));
        if let Some((next_key, next_item)) = iterators[stream_index].next() {
            heap.push(MergeHead {
                key: Reverse(next_key.clone()),
                stream_index,
            });
            current[stream_index] = Some((next_key, next_item));
        }
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::{canonical_merge_streams, OrderedReductionStream};

    #[test]
    fn canonical_merge_preserves_global_key_order() {
        let merged = canonical_merge_streams(vec![
            OrderedReductionStream::new(vec![(1u64, "a"), (3u64, "c")]),
            OrderedReductionStream::new(vec![(2u64, "b"), (4u64, "d")]),
        ]);

        assert_eq!(
            merged,
            vec![(1u64, "a"), (2u64, "b"), (3u64, "c"), (4u64, "d")]
        );
    }
}
