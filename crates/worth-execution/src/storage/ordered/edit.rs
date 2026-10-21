//! AVL path replacement. Child and entry cells retain their original funding.
use super::{
    node::{build, height, Entry, Link, Node},
    ExecutionOrderedMapDenial as Denial,
};
use crate::ExecutionAllocationPolicy as Policy;
use std::{cmp::Ordering, sync::Arc};
pub(super) fn insert<K: Ord, V>(
    root: &Link<K, V>,
    entry: Arc<Entry<K, V>>,
    policy: Policy<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    let Some(old) = root else {
        return build(entry, None, None, policy);
    };
    match entry.key.cmp(&old.entry.key) {
        Ordering::Equal => build(entry, old.left.clone(), old.right.clone(), policy),
        Ordering::Less => balanced(
            Arc::clone(&old.entry),
            Some(insert(&old.left, entry, policy)?),
            old.right.clone(),
            policy,
        ),
        Ordering::Greater => balanced(
            Arc::clone(&old.entry),
            old.left.clone(),
            Some(insert(&old.right, entry, policy)?),
            policy,
        ),
    }
}
pub(super) fn remove<K: Ord, V>(
    root: &Link<K, V>,
    key: &K,
    policy: Policy<'_, '_>,
) -> Result<Link<K, V>, Denial> {
    let Some(old) = root else { return Ok(None) };
    match key.cmp(&old.entry.key) {
        Ordering::Less => balanced(
            Arc::clone(&old.entry),
            remove(&old.left, key, policy)?,
            old.right.clone(),
            policy,
        )
        .map(Some),
        Ordering::Greater => balanced(
            Arc::clone(&old.entry),
            old.left.clone(),
            remove(&old.right, key, policy)?,
            policy,
        )
        .map(Some),
        Ordering::Equal => match (&old.left, &old.right) {
            (None, _) => Ok(old.right.clone()),
            (_, None) => Ok(old.left.clone()),
            (Some(_), Some(right)) => {
                let mut first = right;
                while let Some(left) = &first.left {
                    first = left;
                }
                balanced(
                    Arc::clone(&first.entry),
                    old.left.clone(),
                    remove(&old.right, &first.entry.key, policy)?,
                    policy,
                )
                .map(Some)
            }
        },
    }
}
fn balanced<K, V>(
    entry: Arc<Entry<K, V>>,
    left: Link<K, V>,
    right: Link<K, V>,
    policy: Policy<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    if height(&left) > height(&right) + 1 {
        let child = left.as_ref().expect("left-heavy node has a left child");
        if height(&child.left) >= height(&child.right) {
            let right = build(entry, child.right.clone(), right, policy)?;
            build(
                Arc::clone(&child.entry),
                child.left.clone(),
                Some(right),
                policy,
            )
        } else {
            let pivot = child.right.as_ref().expect("inner rotation has a pivot");
            let left = build(
                Arc::clone(&child.entry),
                child.left.clone(),
                pivot.left.clone(),
                policy,
            )?;
            let right = build(entry, pivot.right.clone(), right, policy)?;
            build(Arc::clone(&pivot.entry), Some(left), Some(right), policy)
        }
    } else if height(&right) > height(&left) + 1 {
        let child = right.as_ref().expect("right-heavy node has a right child");
        if height(&child.right) >= height(&child.left) {
            let left = build(entry, left, child.left.clone(), policy)?;
            build(
                Arc::clone(&child.entry),
                Some(left),
                child.right.clone(),
                policy,
            )
        } else {
            let pivot = child.left.as_ref().expect("inner rotation has a pivot");
            let left = build(entry, left, pivot.left.clone(), policy)?;
            let right = build(
                Arc::clone(&child.entry),
                pivot.right.clone(),
                child.right.clone(),
                policy,
            )?;
            build(Arc::clone(&pivot.entry), Some(left), Some(right), policy)
        }
    } else {
        build(entry, left, right, policy)
    }
}
