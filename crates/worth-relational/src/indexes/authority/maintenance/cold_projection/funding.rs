//! Fresh im 15 trees own their nodes; cold construction copies no shared prefix.
use super::{Denial, MaintenanceWork};
use crate::identity::data::{EntityId, RelationId};
use crate::indexes::data::ColdJoinLeg;
use std::mem::size_of;

pub(in crate::indexes::authority::maintenance) fn tree_bytes<T>(
    entries: usize,
) -> Result<u64, Denial> {
    let mut size = entries;
    let mut height = 1;
    while size > 64 {
        size = size.div_ceil(32);
        height += 1;
    }
    // Non-root nodes hold at least 31 entries. Two transient split/root
    // nodes per level cover construction; a tree below a split owns one node.
    let nodes = if entries <= 64 {
        1
    } else {
        entries
            .div_ceil(31)
            .checked_add(2 * height)
            .ok_or(Denial::WorkBudgetExceeded)?
    };
    let node = size_of::<T>()
        .checked_mul(64)
        .and_then(|n| n.checked_add(71 * size_of::<usize>()))
        .ok_or(Denial::WorkBudgetExceeded)?;
    nodes
        .checked_mul(node)
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(Denial::WorkBudgetExceeded)
}
pub(super) fn legs(rows: &ColdJoinLeg, work: &mut MaintenanceWork) -> Result<(), Denial> {
    let mut bytes =
        tree_bytes::<(EntityId, im::OrdMap<EntityId, im::OrdSet<RelationId>>)>(rows.len())?;
    let mut count = 0_usize;
    for external in rows.values() {
        bytes = bytes
            .checked_add(tree_bytes::<(EntityId, im::OrdSet<RelationId>)>(
                external.len(),
            )?)
            .ok_or(Denial::WorkBudgetExceeded)?;
        for relations in external.values() {
            count = count
                .checked_add(relations.len())
                .ok_or(Denial::WorkBudgetExceeded)?;
            bytes = bytes
                .checked_add(tree_bytes::<RelationId>(relations.len())?)
                .ok_or(Denial::WorkBudgetExceeded)?;
        }
    }
    bytes = bytes
        .checked_add(tree_bytes::<(RelationId, (EntityId, EntityId))>(count)?)
        .ok_or(Denial::WorkBudgetExceeded)?;
    let levels = u64::from(usize::BITS - count.leading_zeros()) + 1;
    work.prepare(
        (count as u64)
            .checked_mul(levels * 256)
            .ok_or(Denial::WorkBudgetExceeded)?,
        bytes,
    )
}
