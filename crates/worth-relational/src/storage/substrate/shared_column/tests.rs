use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
struct Counted {
    value: usize,
    copies: Arc<AtomicUsize>,
}

impl Clone for Counted {
    fn clone(&self) -> Self {
        self.copies.fetch_add(1, Ordering::Relaxed);
        Self {
            value: self.value,
            copies: self.copies.clone(),
        }
    }
}

#[test]
fn adoption_counts_actual_path_copies_without_cloning_values() {
    let copies = Arc::new(AtomicUsize::new(0));
    let original: SharedColumn<_> = (0..1024)
        .map(|value| Counted {
            value,
            copies: copies.clone(),
        })
        .collect();
    let mut changed = original.clone();
    let node_bytes = std::mem::size_of::<ColumnNode<Counted>>() as u64;
    let page_bytes = (PAGE_LEN
        * std::mem::size_of::<Option<crate::storage::substrate::StorageAllocation<Counted>>>())
        as u64;
    assert_eq!(
        changed.copy_value_from(0, &original, 1),
        6 * node_bytes + page_bytes
    );
    assert_eq!(changed.copy_value_from(1, &original, 2), 0);
    assert_eq!(
        changed.copy_value_from(32, &original, 33),
        node_bytes + page_bytes
    );
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    assert_eq!(original[0].value, 0);
    assert_eq!(changed[0].value, 1);
}

#[test]
fn retained_columns_copy_only_the_mutated_value_across_page_and_tree_boundaries() {
    let copies = Arc::new(AtomicUsize::new(0));
    let original: SharedColumn<_> = (0..10_000)
        .map(|value| Counted {
            value,
            copies: copies.clone(),
        })
        .collect();
    let mut changed = original.clone();
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    for index in [0, 31, 32, 63, 64, 8191, 8192, 9999] {
        changed[index].value = index + 10_000;
        assert_eq!(original[index].value, index);
    }
    assert_eq!(copies.load(Ordering::Relaxed), 8);
    changed.push(Counted {
        value: 10_000,
        copies: copies.clone(),
    });
    assert_eq!(copies.load(Ordering::Relaxed), 8);
    assert_eq!(original.len(), 10_000);
    assert_eq!(changed.len(), 10_001);
    assert_eq!(
        original.iter().map(|value| value.value).collect::<Vec<_>>(),
        (0..10_000).collect::<Vec<_>>()
    );
    drop(original);
    changed[500].value = 50_000;
    assert_eq!(copies.load(Ordering::Relaxed), 8);
}

#[test]
fn serialization_preserves_sequence_meaning_and_bounds() {
    for len in [0, 1, 31, 32, 33, 64, 65, 1025] {
        let flat: Vec<u64> = (0..len).collect();
        let column: SharedColumn<_> = flat.clone().into();
        assert_eq!(
            rmp_serde::to_vec(&column).unwrap(),
            rmp_serde::to_vec(&flat).unwrap()
        );
        assert_eq!(column.get(len as usize), None);
        assert_eq!(column.iter().len(), len as usize);
    }
}

#[test]
fn dense_restore_bootstrap_preserves_pages_and_future_copy_on_write() {
    for len in [0_usize, 1, 31, 32, 33, 63, 64, 65, 1025, 8192] {
        let flat = (0..len).collect::<Vec<_>>();
        let column: SharedColumn<_> = flat.clone().into();
        assert_eq!(column.iter().copied().collect::<Vec<_>>(), flat);
        assert_eq!(column.iter().rev().copied().collect::<Vec<_>>(), {
            let mut reversed = flat.clone();
            reversed.reverse();
            reversed
        });
        assert_eq!(
            column.height,
            len.div_ceil(PAGE_LEN).next_power_of_two().trailing_zeros() as usize
        );
        if let Some(root) = column.root.as_ref() {
            let mut level = len.div_ceil(PAGE_LEN);
            let mut expected_nodes = 0;
            while level > 0 {
                expected_nodes += level;
                if level == 1 {
                    break;
                }
                level = level.div_ceil(2);
            }
            assert_eq!(root.page_count, len.div_ceil(PAGE_LEN));
            assert_eq!(root.value_count, len);
            assert_eq!(root.node_count, expected_nodes);
        } else {
            assert_eq!(len, 0);
        }
        let mut changed = column.clone();
        changed.push(len);
        assert_eq!(changed[len], len);
        assert_eq!(column.len(), len);
        if len > 0 {
            changed[len / 2] = len + 1;
            assert_eq!(column[len / 2], len / 2);
            assert_eq!(changed[len / 2], len + 1);
        }
    }
}

#[test]
fn dense_restore_bootstrap_moves_values_without_cloning_them() {
    let copies = Arc::new(AtomicUsize::new(0));
    let values = (0..1025)
        .map(|value| Counted {
            value,
            copies: copies.clone(),
        })
        .collect::<Vec<_>>();
    let column: SharedColumn<_> = values.into();
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    let restored = column.into_vec();
    assert_eq!(restored.len(), 1025);
    assert_eq!(copies.load(Ordering::Relaxed), 0);
    assert_eq!(restored[1024].value, 1024);
}

#[test]
#[ignore = "run explicitly to measure synthetic dense column restoration"]
fn synthetic_dense_column_bootstrap_benchmark() {
    const VALUES: usize = 100_000;
    const SAMPLES: usize = 5;
    let values = (0..VALUES).collect::<Vec<_>>();
    let measure = |bulk: bool| {
        let inputs = (0..SAMPLES).map(|_| values.clone()).collect::<Vec<_>>();
        let started = std::time::Instant::now();
        for input in inputs {
            let input = std::hint::black_box(input);
            let column: SharedColumn<_> = if bulk {
                input.into()
            } else {
                input.into_iter().collect()
            };
            std::hint::black_box(column.len());
        }
        started.elapsed()
    };
    let scalar = measure(false);
    let bulk = measure(true);
    eprintln!(
        "synthetic dense SharedColumn bootstrap: values={VALUES} samples={SAMPLES} scalar={scalar:?} bulk={bulk:?}"
    );
}

#[test]
fn sparse_shapes_allocate_only_selected_pages_and_preserve_sequence_export() {
    for len in [0, 1, 31, 32, 33, 64, 65, 1025, 10_000] {
        let mut column = SharedColumn::with_default(len, 0u64);
        let mut flat = vec![0u64; len];
        assert!(column.root.is_none());
        let retained = column.clone();
        for index in [0, 31, 32, 63, 64, 8191, 8192, 9999]
            .into_iter()
            .filter(|&i| i < len)
        {
            column.set(index, index as u64 + 1);
            flat[index] = index as u64 + 1;
            assert_eq!(retained[index], 0);
        }
        column.push(42);
        flat.push(42);
        assert_eq!(column.to_vec(), flat);
        assert_eq!(column.clone().into_vec(), flat);
        assert_eq!(
            rmp_serde::to_vec(&column).unwrap(),
            rmp_serde::to_vec(&flat).unwrap()
        );
        assert_eq!(retained.iter().copied().sum::<u64>(), 0);
        assert!(column.root.as_ref().unwrap().page_count <= 9);
    }
}

#[test]
fn mixed_direction_iteration_and_retention_preserve_order_and_snapshot() {
    for len in [0, 1, 31, 32, 33, 1025] {
        for sparse in [false, true] {
            let mut column: SharedColumn<u64> = if sparse {
                SharedColumn::with_default(len, 0)
            } else {
                (0..len as u64).collect()
            };
            let expected = column.to_vec();
            let mut flat = expected.iter();
            let mut borrowed = column.iter();
            for i in 0..len {
                if i % 3 == 0 {
                    assert_eq!(borrowed.next_back(), flat.next_back());
                } else {
                    assert_eq!(borrowed.next(), flat.next());
                }
                assert_eq!(borrowed.len(), flat.len());
            }
            assert_eq!(borrowed.next(), None);
            assert_eq!(borrowed.next_back(), None);
            let retained = column.clone();
            column.retain(|value| value % 3 != 0);
            assert_eq!(
                column.to_vec(),
                expected
                    .iter()
                    .copied()
                    .filter(|value| value % 3 != 0)
                    .collect::<Vec<_>>()
            );
            assert_eq!(retained.to_vec(), expected);
            assert_eq!(
                retained.partition_point(|value| *value < 17),
                expected.partition_point(|value| *value < 17)
            );
        }
    }
}
