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
pub(super) fn detached_bytes<K, V>(entry: &Entry<K, V>) -> Result<Option<u64>, Denial> {
    entry
        .bytes
        .map(|n| {
            n.checked_add(arc_bytes::<Node<K, V>>()?)
                .ok_or(Denial::CapacityOverflow)
        })
        .transpose()
}
/// Both original owners are admitted before either allocation is made.
pub(super) fn prepared_copy<K: Copy, V: Copy>(
    key: K,
    value: V,
    policy: Funding<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    prepared_owned(key, value, policy)
}

/// Move native payload custody without invoking its Clone implementation.
pub(super) fn prepared_owned<K, V>(
    key: K,
    value: V,
    policy: Funding<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    let (entry_bytes, node_bytes) = if policy.uses_accounting() {
        (
            Some(arc_bytes::<Entry<K, V>>()?),
            Some(arc_bytes::<Node<K, V>>()?),
        )
    } else {
        (None, None)
    };
    let bytes = match (entry_bytes, node_bytes) {
        (Some(entry), Some(node)) => Some(entry.checked_add(node).ok_or(Denial::CapacityOverflow)?),
        _ => None,
    };
    let entry_memory = entry_bytes
        .map(|n| policy.reserve_memory(n))
        .transpose()?
        .flatten();
    let node_memory = node_bytes
        .map(|n| policy.reserve_memory(n))
        .transpose()?
        .flatten();
    let entry = Arc::new(Entry {
        key,
        value,
        bytes: entry_bytes,
        _memory: entry_memory,
    });
    Ok(Arc::new(Node {
        entry,
        left: None,
        right: None,
        height: 1,
        count: 1,
        bytes,
        _memory: node_memory,
    }))
}

/// Refresh a moved, uniquely owned cell without creating accounting knowledge
/// for a node that previously had none. Its allocation and grant never move apart.
pub(super) fn refresh<K, V>(node: &mut Node<K, V>) {
    node.height = height(&node.left).max(height(&node.right)) + 1;
    node.count = node
        .left
        .as_ref()
        .map_or(0, |n| n.count)
        .checked_add(node.right.as_ref().map_or(0, |n| n.count))
        .and_then(|n| n.checked_add(1))
        .expect("unique edit preflights total addressable cardinality");
    node.bytes = match (
        node.bytes,
        node.entry.bytes,
        node.left.as_ref().map_or(Some(0), |n| n.bytes),
        node.right.as_ref().map_or(Some(0), |n| n.bytes),
    ) {
        (Some(_), Some(entry), Some(left), Some(right)) => Some(
            arc_bytes::<Node<K, V>>()
                .expect("existing node layout remains valid")
                .checked_add(entry)
                .and_then(|n| n.checked_add(left))
                .and_then(|n| n.checked_add(right))
                .expect("unique edit preflights total retained bytes"),
        ),
        _ => None,
    };
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
impl<K, V> Entry<K, V> {
    pub(super) fn owned_native(
        key: K,
        value: V,
        policy: Funding<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let bytes = policy
            .uses_accounting()
            .then(arc_bytes::<Self>)
            .transpose()?;
        let memory = bytes
            .map(|bytes| policy.reserve_memory(bytes))
            .transpose()?
            .flatten();
        Ok(Arc::new(Self {
            key,
            value,
            bytes,
            _memory: memory,
        }))
    }
}
pub(super) fn height<K, V>(link: &Link<K, V>) -> u8 {
    link.as_ref().map_or(0, |n| n.height)
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
