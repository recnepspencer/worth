use super::{funding::Funding, ExecutionOrderedMapDenial as Denial};
use crate::{ChargedBytes, ExecutionMemoryReservation};
use std::{alloc::Layout, sync::Arc};
pub(super) type Link<K, V> = Option<Arc<Node<K, V>>>;
pub(super) struct Entry<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    bytes: Option<u64>,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(super) struct Node<K, V> {
    pub(super) entry: Arc<Entry<K, V>>,
    pub(super) left: Link<K, V>,
    pub(super) right: Link<K, V>,
    pub(super) height: u8,
    pub(super) count: usize,
    pub(super) bytes: Option<u64>,
    _memory: Option<ExecutionMemoryReservation>,
}
fn arc_bytes<T>() -> Result<u64, Denial> {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or(Denial::CapacityOverflow)
}
pub(crate) fn cell_bytes<K, V>() -> Result<u64, Denial> {
    arc_bytes::<Entry<K, V>>()?
        .checked_add(arc_bytes::<Node<K, V>>()?)
        .ok_or(Denial::CapacityOverflow)
}
impl<K: Clone + ChargedBytes, V: Clone + ChargedBytes> Entry<K, V> {
    pub(super) fn cloned(key: &K, value: &V, policy: Funding<'_, '_>) -> Result<Arc<Self>, Denial> {
        if !policy.uses_accounting() {
            return Self::cloned_with_bytes(key, value, policy, None);
        }
        let bytes = arc_bytes::<Self>()?
            .checked_add(key.additional_charged_bytes())
            .and_then(|n| n.checked_add(value.additional_charged_bytes()))
            .ok_or(Denial::CapacityOverflow)?;
        Self::cloned_with_bytes(key, value, policy, Some(bytes))
    }
}
impl<K: Clone, V: Clone> Entry<K, V> {
    pub(super) fn cloned_native(
        key: &K,
        value: &V,
        policy: Funding<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let bytes = if policy.uses_accounting() {
            Some(arc_bytes::<Self>()?)
        } else {
            None
        };
        Self::cloned_with_bytes(key, value, policy, bytes)
    }
    fn cloned_with_bytes(
        key: &K,
        value: &V,
        policy: Funding<'_, '_>,
        bytes: Option<u64>,
    ) -> Result<Arc<Self>, Denial> {
        let memory = bytes
            .map(|bytes| policy.reserve_memory(bytes))
            .transpose()?
            .flatten();
        let entry = Self {
            key: key.clone(),
            value: value.clone(),
            bytes,
            _memory: memory,
        };
        Ok(Arc::new(entry))
    }
}
pub(super) fn height<K, V>(link: &Link<K, V>) -> u8 {
    link.as_ref().map_or(0, |n| n.height)
}

#[derive(Clone, Copy)]
pub(super) enum Rotation {
    None,
    Right,
    LeftRight,
    Left,
    RightLeft,
}

/// Both persistent and uniquely-owned edits use this one AVL decision.
pub(super) fn rotation<K, V>(left: &Link<K, V>, right: &Link<K, V>) -> Rotation {
    if height(left) > height(right) + 1 {
        let child = left.as_ref().expect("left-heavy node");
        if height(&child.left) >= height(&child.right) {
            Rotation::Right
        } else {
            Rotation::LeftRight
        }
    } else if height(right) > height(left) + 1 {
        let child = right.as_ref().expect("right-heavy node");
        if height(&child.right) >= height(&child.left) {
            Rotation::Left
        } else {
            Rotation::RightLeft
        }
    } else {
        Rotation::None
    }
}

/// Removal only rearranges existing cells or removes them. Every new subtree
/// is a subset of the already-representable tree; no new byte quote is made.
pub(super) fn refresh_owned<K, V>(node: &mut Node<K, V>) {
    node.height = height(&node.left)
        .max(height(&node.right))
        .checked_add(1)
        .expect("an existing representable AVL height");
    node.count = node
        .left
        .as_ref()
        .map_or(0, |n| n.count)
        .checked_add(node.right.as_ref().map_or(0, |n| n.count))
        .and_then(|n| n.checked_add(1))
        .expect("a subset of the admitted tree");
    node.bytes = match (
        node.bytes,
        node.entry.bytes,
        node.left.as_ref().map_or(Some(0), |n| n.bytes),
        node.right.as_ref().map_or(Some(0), |n| n.bytes),
    ) {
        (Some(_), Some(entry), Some(left), Some(right)) => Some(
            arc_bytes::<Node<K, V>>()
                .expect("existing node layout")
                .checked_add(entry)
                .and_then(|n| n.checked_add(left))
                .and_then(|n| n.checked_add(right))
                .expect("a subset of the admitted backing"),
        ),
        _ => None,
    };
}
pub(super) fn build<K, V>(
    entry: Arc<Entry<K, V>>,
    left: Link<K, V>,
    right: Link<K, V>,
    policy: Funding<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    let allocation = if policy.uses_accounting() {
        Some(arc_bytes::<Node<K, V>>()?)
    } else {
        None
    };
    let count = left
        .as_ref()
        .map_or(0, |n| n.count)
        .checked_add(right.as_ref().map_or(0, |n| n.count))
        .and_then(|n| n.checked_add(1))
        .ok_or(Denial::CapacityOverflow)?;
    let bytes = match (
        allocation,
        entry.bytes,
        left.as_ref().map_or(Some(0), |n| n.bytes),
        right.as_ref().map_or(Some(0), |n| n.bytes),
    ) {
        (Some(allocation), Some(entry), Some(left), Some(right)) => Some(
            allocation
                .checked_add(entry)
                .and_then(|n| n.checked_add(left))
                .and_then(|n| n.checked_add(right))
                .ok_or(Denial::CapacityOverflow)?,
        ),
        _ => None,
    };
    let level = height(&left)
        .max(height(&right))
        .checked_add(1)
        .ok_or(Denial::CapacityOverflow)?;
    let memory = allocation
        .map(|bytes| policy.reserve_memory(bytes))
        .transpose()?
        .flatten();
    Ok(Arc::new(Node {
        entry,
        left,
        right,
        height: level,
        count,
        bytes,
        _memory: memory,
    }))
}
