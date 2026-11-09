use super::{funding::PathFunding, ExecutionOrderedMapDenial as Denial};
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
pub(super) fn arc_bytes<T>() -> Result<u64, Denial> {
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
/// Both original cell owners are admitted before either allocation is made.
pub(super) fn prepared_copy<K: Copy, V: Copy>(
    key: K,
    value: V,
    source: &dyn super::funding::FundingSource,
) -> Result<Arc<Node<K, V>>, Denial> {
    prepared_owned(key, value, source)
}

/// Move native payload custody through the same admitted path funding owner.
pub(super) fn prepared_owned<K, V>(
    key: K,
    value: V,
    source: &dyn super::funding::FundingSource,
) -> Result<Arc<Node<K, V>>, Denial> {
    let funding = PathFunding::admit(source, cell_bytes::<K, V>()?)?;
    let entry = Entry::owned_native(key, value, &funding)?;
    build(entry, None, None, &funding)
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
pub(super) struct EntryPlan<'a, K, V> {
    key: &'a K,
    value: &'a V,
    bytes: Option<u64>,
}
impl<'a, K: Clone, V: Clone> EntryPlan<'a, K, V> {
    pub(super) fn native(key: &'a K, value: &'a V, accounted: bool) -> Result<Self, Denial> {
        Ok(Self {
            key,
            value,
            bytes: accounted.then(arc_bytes::<Entry<K, V>>).transpose()?,
        })
    }
    pub(super) fn bytes(&self) -> u64 {
        self.bytes.unwrap_or(0)
    }
    pub(super) fn seal(self, pool: &PathFunding) -> Result<Arc<Entry<K, V>>, Denial> {
        let memory = self
            .bytes
            .map(|bytes| pool.reserve_memory(bytes))
            .transpose()?
            .flatten();
        Ok(Arc::new(Entry {
            key: self.key.clone(),
            value: self.value.clone(),
            bytes: self.bytes,
            _memory: memory,
        }))
    }
}
impl<'a, K: Clone + ChargedBytes, V: Clone + ChargedBytes> EntryPlan<'a, K, V> {
    pub(super) fn measured(key: &'a K, value: &'a V, accounted: bool) -> Result<Self, Denial> {
        let mut plan = Self::native(key, value, accounted)?;
        if let Some(bytes) = plan.bytes {
            plan.bytes = Some(
                bytes
                    .checked_add(key.additional_charged_bytes())
                    .and_then(|n| n.checked_add(value.additional_charged_bytes()))
                    .ok_or(Denial::CapacityOverflow)?,
            );
        }
        Ok(plan)
    }
}
impl<K, V> Entry<K, V> {
    pub(super) fn owned_native(
        key: K,
        value: V,
        policy: &PathFunding,
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
    policy: &PathFunding,
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
