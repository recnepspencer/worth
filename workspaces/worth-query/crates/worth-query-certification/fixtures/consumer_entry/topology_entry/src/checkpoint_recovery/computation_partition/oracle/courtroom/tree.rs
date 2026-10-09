//! Interpret Cartesian shape and aggregate cutoff from model leaves only.
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::PartitionIdentity as Id;

struct Node {
    key: Id,
    value: f64,
    aggregate: f64,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn build(leaves: &[(Id, f64)]) -> Option<Box<Self>> {
        let root = leaves
            .iter()
            .enumerate()
            .min_by_key(|(_, (key, _))| tree_work::priority(*key))?
            .0;
        let left = Self::build(&leaves[..root]);
        let right = Self::build(&leaves[root + 1..]);
        let (key, value) = leaves[root];
        let aggregate = (left.as_ref().map_or(-0.0, |node| node.aggregate) + value)
            + right.as_ref().map_or(-0.0, |node| node.aggregate);
        Some(Box::new(Self {
            key,
            value,
            aggregate,
            left,
            right,
        }))
    }

    fn path<'a>(&'a self, key: Id, path: &mut Vec<&'a Self>) {
        path.push(self);
        if key != self.key {
            let child = if key < self.key {
                &self.left
            } else {
                &self.right
            };
            child.as_ref().unwrap().path(key, path);
        }
    }
}

fn leaves(model: &Model) -> BTreeMap<Id, f64> {
    model
        .partition_values()
        .into_iter()
        .map(|(key, value)| (tree_work::identity(key), value))
        .collect()
}

pub(super) fn expected_nodes(now: &Model, prior: Option<&Model>) -> u128 {
    if !now.completes() {
        return 0;
    }
    let desired = leaves(now);
    let Some(prior) = prior.filter(|prior| prior.odd == now.odd) else {
        return desired.len() as u128;
    };
    let mut current = leaves(prior);
    let keys = current
        .keys()
        .chain(desired.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let mut count = 0;
    for key in keys {
        let old_keys = current.keys().copied().collect::<Vec<_>>();
        match (current.get(&key).copied(), desired.get(&key).copied()) {
            (Some(_), None) => {
                count += u128::from(tree_work::delete_bound(&old_keys, key) - 1);
                current.remove(&key);
            }
            (None, Some(value)) => {
                count += u128::from(tree_work::insert_bound(&old_keys, key));
                current.insert(key, value);
            }
            (Some(old), Some(value)) if old.to_bits() != value.to_bits() => {
                let before =
                    Node::build(&current.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>())
                        .unwrap();
                current.insert(key, value);
                let after = Node::build(&current.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>())
                    .unwrap();
                let (mut old_path, mut new_path) = (Vec::new(), Vec::new());
                before.path(key, &mut old_path);
                after.path(key, &mut new_path);
                let mut child_changed = true;
                for (old, new) in old_path.into_iter().zip(new_path).rev() {
                    assert_eq!(old.key, new.key);
                    assert!(old.value.is_finite() && new.value.is_finite());
                    if child_changed {
                        count += 1;
                    }
                    child_changed &= old.aggregate.to_bits() != new.aggregate.to_bits();
                }
            }
            _ => {}
        }
    }
    count
}
