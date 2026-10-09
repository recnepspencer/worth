//! Independent Cartesian-shape oracle: choose the minimum priority recursively,
//! rather than execution's monotone-stack construction or editing traversal.

use worth_foundational::facade::PartitionIdentity as Id;

pub(super) struct Shape {
    key: Id,
    left: Option<Box<Shape>>,
    right: Option<Box<Shape>>,
}

impl Shape {
    pub(super) fn from_sorted(keys: &[Id]) -> Option<Box<Self>> {
        let root = keys
            .iter()
            .enumerate()
            .min_by_key(|(_, key)| priority(**key))?
            .0;
        Some(Box::new(Self {
            key: keys[root],
            left: Self::from_sorted(&keys[..root]),
            right: Self::from_sorted(&keys[root + 1..]),
        }))
    }

    pub(super) fn build_charge(root: &Option<Box<Self>>) -> u64 {
        fn nodes(root: &Option<Box<Shape>>) -> u64 {
            root.as_ref()
                .map_or(0, |n| 1 + nodes(&n.left) + nodes(&n.right))
        }
        fn spine(root: &Option<Box<Shape>>, left: bool) -> u64 {
            root.as_ref().map_or(0, |n| {
                1 + spine(if left { &n.left } else { &n.right }, left)
            })
        }
        // Declared full-tree charge: nine visits per node, minus both spines.
        9 * nodes(root) - spine(root, true) - spine(root, false)
    }

    pub(super) fn update_path(root: &Option<Box<Self>>, key: Id) -> Vec<Id> {
        let mut path = Vec::new();
        let mut current = root.as_deref();
        while let Some(node) = current {
            path.push(node.key);
            if node.key == key {
                return path;
            }
            current = if key < node.key {
                node.left.as_deref()
            } else {
                node.right.as_deref()
            };
        }
        panic!("updated identity is present");
    }

    pub(super) fn depth(root: &Option<Box<Self>>, key: Id) -> u64 {
        let Some(node) = root else { return 0 };
        1 + if key == node.key {
            0
        } else {
            Self::depth(
                if key < node.key {
                    &node.left
                } else {
                    &node.right
                },
                key,
            )
        }
    }

    pub(super) fn rotations(root: &Option<Box<Self>>, key: Id) -> u64 {
        let Some(node) = root else { return 0 };
        u64::from(priority(key) < priority(node.key))
            + Self::rotations(
                if key < node.key {
                    &node.left
                } else {
                    &node.right
                },
                key,
            )
    }

    pub(super) fn deletion_depth(root: &Option<Box<Self>>, key: Id) -> u64 {
        let node = root.as_ref().expect("deleted identity is present");
        if key == node.key {
            1 + Self::merge_depth(&node.left, &node.right)
        } else {
            1 + Self::deletion_depth(
                if key < node.key {
                    &node.left
                } else {
                    &node.right
                },
                key,
            )
        }
    }

    fn merge_depth(left: &Option<Box<Self>>, right: &Option<Box<Self>>) -> u64 {
        Self::spine_merge_depth(left.as_deref(), right.as_deref())
    }

    fn spine_merge_depth(left: Option<&Self>, right: Option<&Self>) -> u64 {
        match (left, right) {
            (Some(l), Some(r)) if priority(l.key) < priority(r.key) => {
                1 + Self::spine_merge_depth(l.right.as_deref(), right)
            }
            (Some(_), Some(r)) => 1 + Self::spine_merge_depth(left, r.left.as_deref()),
            _ => 0,
        }
    }
}

/// The specified portable priority rule, independently evaluated by the oracle.
pub(super) fn priority(key: Id) -> (u64, Id) {
    let mut x = key.value().wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (x ^ (x >> 31), key)
}
