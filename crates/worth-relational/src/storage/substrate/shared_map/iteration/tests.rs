use super::*;
use crate::storage::substrate::SharedMap;
use std::collections::BTreeMap;

#[test]
fn inline_stack_checked_walk_matches_forward_and_exact_size() {
    for len in [0, 1, 2, 3, 31, 128, 1025] {
        let original: SharedMap<u64, u64> = (0..len).rev().map(|key| (key, key + 17)).collect();
        let retained = original.clone();
        let mut changed = original;
        changed.insert(len + 1, 91);
        for map in [&retained, &changed] {
            let expected = (0..len)
                .map(|key| (key, key + 17))
                .chain((map.len() > len as usize).then_some((len + 1, 91)))
                .collect::<BTreeMap<_, _>>();
            let mut plain = map.iter();
            let mut checked = map
                .try_iter(|work, bytes| {
                    assert_eq!(
                        work,
                        map.root.as_ref().map_or(1, |node| node.height as u64 + 1)
                    );
                    assert_eq!(bytes, 0);
                    Ok::<_, ()>(())
                })
                .unwrap();
            for (index, entry) in expected.iter().enumerate() {
                assert_eq!(plain.len(), expected.len() - index);
                assert_eq!(checked.size_hint(), plain.size_hint());
                assert_eq!(plain.next(), Some(entry));
                assert_eq!(
                    checked
                        .try_next(|work, bytes| {
                            assert!(work > 0);
                            assert_eq!(bytes, 0);
                            Ok::<_, ()>(())
                        })
                        .unwrap(),
                    Some(entry)
                );
            }
            assert_eq!(plain.len(), 0);
            assert_eq!(plain.next(), None);
            assert_eq!(
                checked.try_next::<()>(|_, _| panic!("terminal iteration does not poll")),
                Ok(None)
            );
            for probe in [0, len / 2, len, len + 2] {
                assert_eq!(map.lower_bound(&probe), expected.range(probe..).next());
            }
        }
    }
}

fn original_step_prices(node: Option<&MapNode<u64, u64>>, out: &mut Vec<(u64, u64, u64)>) {
    if let Some(node) = node {
        original_step_prices(node.left.as_deref(), out);
        out.push((
            node.key,
            node.value,
            node.right
                .as_ref()
                .map_or(1, |right| right.height as u64 + 1),
        ));
        original_step_prices(node.right.as_deref(), out);
    }
}

#[test]
fn inline_stack_stop_preserves_next_entry_and_original_work_sequence() {
    for len in [0, 1, 9, 257] {
        let map: SharedMap<u64, u64> = (0..len).map(|key| (key, key * 3)).collect();
        let opening = map.root.as_ref().map_or(1, |node| node.height as u64 + 1);
        let refused = map.try_iter(|work, bytes| {
            assert_eq!((work, bytes), (opening, 0));
            Err("original opening stop")
        });
        assert!(matches!(refused, Err("original opening stop")));
        let mut cursor = map
            .try_iter(|work, bytes| {
                assert_eq!((work, bytes), (opening, 0));
                Ok::<_, &str>(())
            })
            .unwrap();
        let mut expected = Vec::new();
        original_step_prices(map.root.as_deref(), &mut expected);
        for (key, value, price) in expected {
            let before = cursor.size_hint();
            assert_eq!(
                cursor.try_next(|work, bytes| {
                    assert_eq!((work, bytes), (price, 0));
                    Err("original step stop")
                }),
                Err("original step stop")
            );
            assert_eq!(cursor.size_hint(), before);
            let next = cursor
                .try_next(|work, bytes| {
                    assert_eq!((work, bytes), (price, 0));
                    Ok::<_, &str>(())
                })
                .unwrap()
                .unwrap();
            assert_eq!(next, (&key, &value));
            assert!(std::ptr::eq(next.1, map.get(&key).unwrap()));
        }
        assert_eq!(cursor.len(), 0);
        assert_eq!(
            cursor.try_next::<()>(|_, _| panic!("terminal iteration does not poll")),
            Ok(None)
        );
    }
}

#[test]
fn inline_stack_nested_walks_report_zero_heap_and_actual_frame_size() {
    let inner: SharedMap<u64, u64> = [(2, 20), (7, 70), (9, 90)].into_iter().collect();
    let outer: SharedMap<u64, SharedMap<u64, u64>> =
        [(1, inner.clone()), (4, inner)].into_iter().collect();
    let mut outer_cursor = outer
        .try_iter(|_, bytes| {
            assert_eq!(bytes, 0);
            Ok::<_, ()>(())
        })
        .unwrap();
    let first = outer_cursor.next().unwrap();
    let mut inner_cursor = first
        .1
        .try_iter(|_, bytes| {
            assert_eq!(bytes, 0);
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(inner_cursor.next(), Some((&2, &20)));
    assert_eq!(outer_cursor.next().unwrap().0, &4);
    assert_eq!(inner_cursor.next(), Some((&7, &70)));
    let empty = SharedMap::<u64, u64>::new();
    let empty_cursor = empty.iter();
    let bytes = std::mem::size_of_val(&empty_cursor);
    assert_eq!(
        bytes,
        std::mem::size_of_val(&empty_cursor.pending) + 2 * std::mem::size_of::<usize>()
    );
    assert_eq!(empty_cursor.pending.len(), 2 * usize::BITS as usize);
    assert_eq!(
        std::mem::size_of::<Option<&MapNode<u64, u64>>>(),
        std::mem::size_of::<usize>()
    );
    let nested_bytes = std::mem::size_of_val(&(outer_cursor, inner_cursor));
    eprintln!("INLINE_MAP_ITERATOR_LAYOUT iterator_bytes={bytes} two_nested_cursors_bytes={nested_bytes} pointer_bits={}", usize::BITS);
    assert_eq!(nested_bytes, 2 * bytes);
}
