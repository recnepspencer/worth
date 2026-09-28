use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::super::ordered_map::{node_len, Link};
use super::super::UiPersistentOrdMap;

/// A row that counts every copy made of it.
#[derive(Debug, Default)]
struct Counted(Rc<Cell<usize>>);

impl Clone for Counted {
    fn clone(&self) -> Self {
        self.0.set(self.0.get() + 1);
        Self(Rc::clone(&self.0))
    }
}

#[test]
fn take_and_edit_copy_a_row_only_while_a_fork_shares_it() {
    let copies = Rc::new(Cell::new(0));
    let mut map = UiPersistentOrdMap::default();
    for key in 0..64 {
        map.insert(key, Counted(Rc::clone(&copies)));
    }
    let fork = map.clone();
    assert!(map.take_with_work(&7).0.is_some());
    assert_eq!(copies.get(), 1, "the row the fork shares is copied out");
    map.edit_or_default(8, |_| {});
    assert_eq!(copies.get(), 2, "an edit copies the row the fork shares");
    assert!(fork.get(&7).is_some() && fork.get(&8).is_some());
    assert!(map.get(&7).is_none() && map.get(&8).is_some());

    drop(fork);
    assert!(map.take_with_work(&9).0.is_some());
    map.edit_or_default(10, |_| {});
    assert_eq!(copies.get(), 2, "unshared rows move out and back uncopied");
    assert!(map.take_with_work(&9).0.is_none());
    assert_eq!(map.len(), 62);
}

#[test]
fn edit_or_default_starts_absent_rows_from_default_and_keeps_forks() {
    let mut map: UiPersistentOrdMap<u32, Vec<u32>> = UiPersistentOrdMap::default();
    map.edit_or_default(3, |rows| rows.push(1));
    let fork = map.clone();
    map.edit_or_default(3, |rows| rows.push(2));
    map.edit_or_default(4, |rows| rows.push(9));
    assert_eq!(map.get(&3), Some(&vec![1, 2]));
    assert_eq!(map.get(&4), Some(&vec![9]));
    assert_eq!(fork.get(&3), Some(&vec![1]));
    assert!(fork.get(&4).is_none());
}

#[test]
fn a_shared_root_answers_equal_and_divergent_forks_compare_rows() {
    let mut map = UiPersistentOrdMap::default();
    for key in 0..32 {
        map.insert(key, key);
    }
    let mut fork = map.clone();
    assert_eq!(map, fork);
    fork.insert(5, 50);
    assert_ne!(map, fork);
    fork.insert(5, 5);
    assert_eq!(map, fork, "rows are compared once the roots differ");
    let mut nan = UiPersistentOrdMap::default();
    nan.insert(0, f32::NAN);
    assert_eq!(nan, nan.clone(), "a shared root skips the rows");
    let mut rebuilt = UiPersistentOrdMap::default();
    rebuilt.insert(0, f32::NAN);
    assert_ne!(nan, rebuilt);
}

#[test]
fn allocations_happen_only_while_a_fork_shares_the_path() {
    let mut map = UiPersistentOrdMap::default();
    for key in 0..128 {
        map.insert(key, key);
    }
    let root = |map: &UiPersistentOrdMap<i32, i32>| Rc::as_ptr(map.root.as_ref().unwrap());
    let unshared = root(&map);
    let replace = map.insert_with_work(0, 1000);
    assert_eq!(
        root(&map),
        unshared,
        "an unshared root is rewritten in place"
    );
    assert_eq!(replace.node_copies(), replace.key_probes());
    assert_eq!(replace.node_allocations(), 0);
    assert_eq!(
        map.insert_with_work(500, 500).node_allocations(),
        1,
        "only the leaf"
    );
    assert_eq!(map.remove_with_work(&64).1.node_allocations(), 0);

    let fork = map.clone();
    let shared = root(&map);
    let first = map.insert_with_work(1, -1);
    assert_ne!(root(&map), shared, "a shared root is copied");
    assert_eq!(root(&fork), shared);
    assert_eq!(
        first.node_allocations(),
        first.node_copies(),
        "the whole shared path"
    );
    let again = map.insert_with_work(1, -2);
    assert_eq!(
        again.node_allocations(),
        0,
        "the copied path is now this map's own"
    );
    assert_eq!((fork.get(&1), map.get(&1)), (Some(&1), Some(&-2)));
    assert_eq!(fork.get(&0), Some(&1000));
}

#[test]
fn mutation_work_pins_rotation_and_successor_counts() {
    let mut single = UiPersistentOrdMap::default();
    single.insert(1, 1);
    single.insert(2, 2);
    let rotated = single.insert_with_work(3, 3);
    assert_eq!((rotated.key_probes(), rotated.node_copies()), (2, 5));

    let mut double = UiPersistentOrdMap::default();
    double.insert(3, 3);
    double.insert(1, 1);
    let rotated = double.insert_with_work(2, 2);
    assert_eq!((rotated.key_probes(), rotated.node_copies()), (2, 8));
    assert_eq!(rotated.node_allocations(), 1);

    let mut successor = UiPersistentOrdMap::default();
    for key in [2, 1, 3] {
        successor.insert(key, key * 10);
    }
    let (removed, work) = successor.remove_with_work(&2);
    assert!(removed);
    assert_eq!((work.key_probes(), work.node_copies()), (1, 1));
    let entries: Vec<_> = successor
        .iter()
        .map(|(key, value)| (*key, *value))
        .collect();
    assert_eq!(entries, [(1, 10), (3, 30)]);
    assert_balanced(&double.root);
    assert_balanced(&successor.root);
}

/// Random edits checked against a `BTreeMap` model, with forks taken along
/// the way; each fork must keep exactly the rows it saw.
#[test]
fn model_checked_edits_never_change_a_fork() {
    let mut map = UiPersistentOrdMap::default();
    let mut model = BTreeMap::new();
    let mut forks = Vec::new();
    let mut seed = 0x2545_f491_u64;
    for step in 0..6_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let key = i32::try_from((seed >> 33) % 300).unwrap();
        if (seed >> 20).is_multiple_of(3) {
            assert_eq!(map.remove(&key), model.remove(&key).is_some());
        } else {
            map.insert(key, step);
            model.insert(key, step);
        }
        if step % 149 == 0 {
            forks.push((map.clone(), entries(&map)));
        }
        if step % 97 == 0 {
            assert_eq!(
                entries(&map),
                model.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>()
            );
            assert_balanced(&map.root);
        }
    }
    assert_eq!(entries(&map), model.into_iter().collect::<Vec<_>>());
    for (fork, seen) in &forks {
        assert_eq!(&entries(fork), seen);
        assert_balanced(&fork.root);
    }
}

fn entries(map: &UiPersistentOrdMap<i32, i32>) -> Vec<(i32, i32)> {
    map.iter().map(|(key, value)| (*key, *value)).collect()
}

/// Checks every node's cached height and length, its AVL balance and its
/// key order, answering the subtree's height.
fn assert_balanced(link: &Link<i32, i32>) -> u16 {
    let Some(node) = link else { return 0 };
    let (left, right) = (assert_balanced(&node.left), assert_balanced(&node.right));
    assert!(left.abs_diff(right) <= 1, "node {} is unbalanced", node.key);
    assert_eq!(node.height, 1 + left.max(right));
    assert_eq!(node.len, 1 + node_len(&node.left) + node_len(&node.right));
    assert!(node.left.as_ref().is_none_or(|child| child.key < node.key));
    assert!(node.right.as_ref().is_none_or(|child| child.key > node.key));
    node.height
}
