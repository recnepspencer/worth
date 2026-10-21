use super::ExecutionOrderedMapDenial as Denial;
use crate::{ChargedBytes, ExecutionAllocationPolicy, ExecutionMemoryReservation};
use std::{alloc::Layout, sync::Arc};
pub(super) type Link<K, V> = Option<Arc<Node<K, V>>>;
pub(super) struct Entry<K, V> {
    pub(super) key: K,
    pub(super) value: V,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(super) struct Node<K, V> {
    pub(super) entry: Arc<Entry<K, V>>,
    pub(super) left: Link<K, V>,
    pub(super) right: Link<K, V>,
    pub(super) height: u8,
    pub(super) count: usize,
    pub(super) bytes: u64,
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
    pub(super) fn cloned(
        key: &K,
        value: &V,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let bytes = arc_bytes::<Self>()?
            .checked_add(key.additional_charged_bytes())
            .and_then(|n| n.checked_add(value.additional_charged_bytes()))
            .ok_or(Denial::CapacityOverflow)?;
        let memory = policy.reserve_memory(bytes)?;
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
pub(super) fn build<K, V>(
    entry: Arc<Entry<K, V>>,
    left: Link<K, V>,
    right: Link<K, V>,
    policy: ExecutionAllocationPolicy<'_, '_>,
) -> Result<Arc<Node<K, V>>, Denial> {
    let allocation = arc_bytes::<Node<K, V>>()?;
    let count = left
        .as_ref()
        .map_or(0, |n| n.count)
        .checked_add(right.as_ref().map_or(0, |n| n.count))
        .and_then(|n| n.checked_add(1))
        .ok_or(Denial::CapacityOverflow)?;
    let bytes = allocation
        .checked_add(entry.bytes)
        .and_then(|n| n.checked_add(left.as_ref().map_or(0, |n| n.bytes)))
        .and_then(|n| n.checked_add(right.as_ref().map_or(0, |n| n.bytes)))
        .ok_or(Denial::CapacityOverflow)?;
    let level = height(&left)
        .max(height(&right))
        .checked_add(1)
        .ok_or(Denial::CapacityOverflow)?;
    let memory = policy.reserve_memory(allocation)?;
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
