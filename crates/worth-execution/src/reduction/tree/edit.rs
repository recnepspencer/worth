use std::{cmp::Ordering, sync::Arc};

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{
    super::plan::{priority, ReductionDenial},
    recombine::Engine,
    Link, ReductionRunStop,
};

pub(super) struct Updated<T> {
    pub(super) link: Link<T>,
    aggregate_changed: bool,
    any_change: bool,
}

pub(super) fn contains<T>(root: &Link<T>, partition: PartitionIdentity) -> bool {
    let mut current = root.as_deref();
    while let Some(node) = current {
        match partition.cmp(&node.identity) {
            Ordering::Less => current = node.left.as_deref(),
            Ordering::Greater => current = node.right.as_deref(),
            Ordering::Equal => return true,
        }
    }
    false
}

pub(super) fn search_depth<T>(root: &Link<T>, partition: PartitionIdentity) -> (usize, bool) {
    let mut depth = 0;
    let mut current = root.as_deref();
    while let Some(node) = current {
        depth += 1;
        match partition.cmp(&node.identity) {
            Ordering::Less => current = node.left.as_deref(),
            Ordering::Greater => current = node.right.as_deref(),
            Ordering::Equal => return (depth, true),
        }
    }
    (depth, false)
}

pub(super) fn deletion_depth<T>(root: &Link<T>, partition: PartitionIdentity) -> Option<usize> {
    let mut depth = 0;
    let mut current = root.as_deref();
    while let Some(node) = current {
        depth += 1;
        match partition.cmp(&node.identity) {
            Ordering::Less => current = node.left.as_deref(),
            Ordering::Greater => current = node.right.as_deref(),
            Ordering::Equal => {
                return Some(depth + merge_depth(&node.left, &node.right));
            }
        }
    }
    None
}

fn merge_depth<T>(left: &Link<T>, right: &Link<T>) -> usize {
    match (left, right) {
        (None, _) | (_, None) => 0,
        (Some(left), Some(right)) if priority(left.identity) < priority(right.identity) => {
            1 + merge_depth(&left.right, &Some(Arc::clone(right)))
        }
        (Some(left), Some(right)) => 1 + merge_depth(&Some(Arc::clone(left)), &right.left),
    }
}

pub(super) fn insert<T, F, H, E>(
    root: &Link<T>,
    partition: PartitionIdentity,
    value: T,
    engine: &mut Engine<'_, T, F, H>,
) -> Result<Link<T>, ReductionRunStop<E>>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    let Some(node) = root else {
        return engine.build(partition, value, None, None).map(Some);
    };
    if partition < node.identity {
        let child = insert(&node.left, partition, value, engine)?;
        if priority(partition) < priority(node.identity)
            && child
                .as_ref()
                .is_some_and(|left| left.identity == partition)
        {
            let top = child.expect("checked child identity");
            let lower = engine.build(
                node.identity,
                node.value.clone(),
                top.right.clone(),
                node.right.clone(),
            )?;
            let rotated = engine.build(
                top.identity,
                top.value.clone(),
                top.left.clone(),
                Some(lower),
            )?;
            return Ok(Some(rotated));
        }
        engine
            .build(node.identity, node.value.clone(), child, node.right.clone())
            .map(Some)
    } else {
        let child = insert(&node.right, partition, value, engine)?;
        if priority(partition) < priority(node.identity)
            && child
                .as_ref()
                .is_some_and(|right| right.identity == partition)
        {
            let top = child.expect("checked child identity");
            let lower = engine.build(
                node.identity,
                node.value.clone(),
                node.left.clone(),
                top.left.clone(),
            )?;
            let rotated = engine.build(
                top.identity,
                top.value.clone(),
                Some(lower),
                top.right.clone(),
            )?;
            return Ok(Some(rotated));
        }
        engine
            .build(node.identity, node.value.clone(), node.left.clone(), child)
            .map(Some)
    }
}

pub(super) fn update<T, F, H, E>(
    root: &Link<T>,
    partition: PartitionIdentity,
    value: T,
    engine: &mut Engine<'_, T, F, H>,
) -> Result<Updated<T>, ReductionRunStop<E>>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    let node = root
        .as_ref()
        .ok_or(ReductionRunStop::Denial(ReductionDenial::UnknownIdentity(
            partition,
        )))?;
    let (next_value, left, right, changed) = match partition.cmp(&node.identity) {
        Ordering::Less => {
            let child = update(&node.left, partition, value, engine)?;
            if !child.any_change {
                return Ok(Updated {
                    link: root.clone(),
                    aggregate_changed: false,
                    any_change: false,
                });
            }
            (
                node.value.clone(),
                child.link,
                node.right.clone(),
                child.aggregate_changed,
            )
        }
        Ordering::Greater => {
            let child = update(&node.right, partition, value, engine)?;
            if !child.any_change {
                return Ok(Updated {
                    link: root.clone(),
                    aggregate_changed: false,
                    any_change: false,
                });
            }
            (
                node.value.clone(),
                node.left.clone(),
                child.link,
                child.aggregate_changed,
            )
        }
        Ordering::Equal => {
            engine.validate(&value)?;
            if engine.same(&node.value, &value)? {
                return Ok(Updated {
                    link: root.clone(),
                    aggregate_changed: false,
                    any_change: false,
                });
            }
            (value, node.left.clone(), node.right.clone(), true)
        }
    };
    let candidate = if changed {
        engine.build(node.identity, next_value, left, right)?
    } else {
        engine.carry(node, next_value, left, right)?
    };
    let aggregate_changed = if changed {
        !engine.same(&node.aggregate, &candidate.aggregate)?
    } else {
        false
    };
    Ok(Updated {
        link: Some(candidate),
        aggregate_changed,
        any_change: true,
    })
}

pub(super) fn delete<T, F, H, E>(
    root: &Link<T>,
    partition: PartitionIdentity,
    engine: &mut Engine<'_, T, F, H>,
) -> Result<Link<T>, ReductionRunStop<E>>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    let node = root
        .as_ref()
        .ok_or(ReductionRunStop::Denial(ReductionDenial::UnknownIdentity(
            partition,
        )))?;
    match partition.cmp(&node.identity) {
        Ordering::Less => {
            let left = delete(&node.left, partition, engine)?;
            engine
                .build(node.identity, node.value.clone(), left, node.right.clone())
                .map(Some)
        }
        Ordering::Greater => {
            let right = delete(&node.right, partition, engine)?;
            engine
                .build(node.identity, node.value.clone(), node.left.clone(), right)
                .map(Some)
        }
        Ordering::Equal => merge(&node.left, &node.right, engine),
    }
}

fn merge<T, F, H, E>(
    left: &Link<T>,
    right: &Link<T>,
    engine: &mut Engine<'_, T, F, H>,
) -> Result<Link<T>, ReductionRunStop<E>>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
    H: FnMut() -> Result<(), E>,
{
    match (left, right) {
        (None, other) | (other, None) => Ok(other.clone()),
        (Some(left), Some(right)) if priority(left.identity) < priority(right.identity) => {
            let merged = merge(&left.right, &Some(Arc::clone(right)), engine)?;
            engine
                .build(left.identity, left.value.clone(), left.left.clone(), merged)
                .map(Some)
        }
        (Some(left), Some(right)) => {
            let merged = merge(&Some(Arc::clone(left)), &right.left, engine)?;
            engine
                .build(
                    right.identity,
                    right.value.clone(),
                    merged,
                    right.right.clone(),
                )
                .map(Some)
        }
    }
}
