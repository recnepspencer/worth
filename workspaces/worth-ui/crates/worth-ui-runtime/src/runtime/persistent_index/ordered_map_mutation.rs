use std::cmp::Ordering;
use std::rc::Rc;

use super::mutation_work::UiPersistentIndexMutationWork;
use super::ordered_map::{height, node_len, Link, Node};

pub(super) fn insert<K: Ord + Clone, V>(
    root: Link<K, V>,
    key: K,
    value: Rc<V>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let Some(node) = root else {
        return make_node(key, value, None, None, work);
    };
    work.record_key_probe();
    match key.cmp(&node.key) {
        Ordering::Less => {
            let left = insert(node.left.clone(), key, value, work);
            balance(
                make_node(
                    node.key.clone(),
                    Rc::clone(&node.value),
                    Some(left),
                    node.right.clone(),
                    work,
                ),
                work,
            )
        }
        Ordering::Greater => {
            let right = insert(node.right.clone(), key, value, work);
            balance(
                make_node(
                    node.key.clone(),
                    Rc::clone(&node.value),
                    node.left.clone(),
                    Some(right),
                    work,
                ),
                work,
            )
        }
        Ordering::Equal => make_node(key, value, node.left.clone(), node.right.clone(), work),
    }
}

impl<K: Ord + Clone, V> super::UiPersistentOrdMap<K, V> {
    /// Applies `edit` to `key`'s value, or to the default when absent. The
    /// value moves out and back, so it is copied only while a fork shares it.
    pub(crate) fn edit_or_default(&mut self, key: K, edit: impl FnOnce(&mut V))
    where
        V: Clone + Default,
    {
        let mut value = self.take_with_work(&key).0.unwrap_or_default();
        edit(&mut value);
        self.insert(key, value);
    }

    /// Removes `key` and hands back its value, copying it only while a fork
    /// still shares it.
    pub(crate) fn take_with_work(&mut self, key: &K) -> (Option<V>, UiPersistentIndexMutationWork)
    where
        V: Clone,
    {
        let mut work = UiPersistentIndexMutationWork::default();
        let (root, removed) = remove(self.root.take(), key, &mut work);
        self.root = root;
        let value =
            removed.map(|value| Rc::try_unwrap(value).unwrap_or_else(|shared| (*shared).clone()));
        (value, work)
    }
}

/// Removes `key`, answering the removed value. The old path is dropped
/// before the caller sees the value, so an unshared value is its only owner.
pub(super) fn remove<K: Ord + Clone, V>(
    root: Link<K, V>,
    key: &K,
    work: &mut UiPersistentIndexMutationWork,
) -> (Link<K, V>, Option<Rc<V>>) {
    let Some(node) = root else {
        return (None, None);
    };
    work.record_key_probe();
    match key.cmp(&node.key) {
        Ordering::Less => {
            let (left, removed) = remove(node.left.clone(), key, work);
            let root = make_node(
                node.key.clone(),
                Rc::clone(&node.value),
                left,
                node.right.clone(),
                work,
            );
            (Some(balance(root, work)), removed)
        }
        Ordering::Greater => {
            let (right, removed) = remove(node.right.clone(), key, work);
            let root = make_node(
                node.key.clone(),
                Rc::clone(&node.value),
                node.left.clone(),
                right,
                work,
            );
            (Some(balance(root, work)), removed)
        }
        Ordering::Equal => match (&node.left, &node.right) {
            (None, _) => (node.right.clone(), Some(Rc::clone(&node.value))),
            (_, None) => (node.left.clone(), Some(Rc::clone(&node.value))),
            (Some(_), Some(right)) => {
                let (successor_key, successor_value, next_right) = take_min(Rc::clone(right), work);
                let root = make_node(
                    successor_key,
                    successor_value,
                    node.left.clone(),
                    next_right,
                    work,
                );
                (Some(balance(root, work)), Some(Rc::clone(&node.value)))
            }
        },
    }
}

fn take_min<K: Ord + Clone, V>(
    node: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> (K, Rc<V>, Link<K, V>) {
    let Some(left) = &node.left else {
        return (node.key.clone(), Rc::clone(&node.value), node.right.clone());
    };
    let (key, value, next_left) = take_min(Rc::clone(left), work);
    let successor = make_node(
        node.key.clone(),
        Rc::clone(&node.value),
        next_left,
        node.right.clone(),
        work,
    );
    (key, value, Some(balance(successor, work)))
}

fn balance<K: Ord + Clone, V>(
    node: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let skew = height(&node.left) as i32 - height(&node.right) as i32;
    if skew > 1 {
        let left = Rc::clone(
            node.left
                .as_ref()
                .expect("left-heavy node has a left child"),
        );
        return if height(&left.left) >= height(&left.right) {
            rotate_right(node, work)
        } else {
            let rotated = rotate_left(left, work);
            let root = with_left(node, rotated, work);
            rotate_right(root, work)
        };
    }
    if skew < -1 {
        let right = Rc::clone(
            node.right
                .as_ref()
                .expect("right-heavy node has a right child"),
        );
        return if height(&right.right) >= height(&right.left) {
            rotate_left(node, work)
        } else {
            let rotated = rotate_right(right, work);
            let root = with_right(node, rotated, work);
            rotate_left(root, work)
        };
    }
    node
}

fn rotate_left<K: Ord + Clone, V>(
    root: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let pivot = root
        .right
        .as_ref()
        .expect("left rotation requires right child");
    let left = make_node(
        root.key.clone(),
        Rc::clone(&root.value),
        root.left.clone(),
        pivot.left.clone(),
        work,
    );
    make_node(
        pivot.key.clone(),
        Rc::clone(&pivot.value),
        Some(left),
        pivot.right.clone(),
        work,
    )
}

fn rotate_right<K: Ord + Clone, V>(
    root: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    let pivot = root
        .left
        .as_ref()
        .expect("right rotation requires left child");
    let right = make_node(
        root.key.clone(),
        Rc::clone(&root.value),
        pivot.right.clone(),
        root.right.clone(),
        work,
    );
    make_node(
        pivot.key.clone(),
        Rc::clone(&pivot.value),
        pivot.left.clone(),
        Some(right),
        work,
    )
}

fn with_left<K: Ord + Clone, V>(
    root: Rc<Node<K, V>>,
    left: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    make_node(
        root.key.clone(),
        Rc::clone(&root.value),
        Some(left),
        root.right.clone(),
        work,
    )
}

fn with_right<K: Ord + Clone, V>(
    root: Rc<Node<K, V>>,
    right: Rc<Node<K, V>>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    make_node(
        root.key.clone(),
        Rc::clone(&root.value),
        root.left.clone(),
        Some(right),
        work,
    )
}

fn make_node<K, V>(
    key: K,
    value: Rc<V>,
    left: Link<K, V>,
    right: Link<K, V>,
    work: &mut UiPersistentIndexMutationWork,
) -> Rc<Node<K, V>> {
    work.record_node_copy();
    Rc::new(Node {
        key,
        value,
        height: 1 + height(&left).max(height(&right)),
        len: 1 + node_len(&left) + node_len(&right),
        left,
        right,
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

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
}
