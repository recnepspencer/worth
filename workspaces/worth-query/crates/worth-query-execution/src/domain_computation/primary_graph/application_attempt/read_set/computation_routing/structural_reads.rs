//! Incremental authoring footprint of one newly attributed structural read.
use super::*;
fn node<T>(len: usize) -> Option<u64> {
    let after = len.checked_add(1)?;
    let levels = (usize::BITS - after.leading_zeros()) as u64 + 1;
    levels.checked_mul(
        11_u64
            .checked_mul(size_of::<T>() as u64)?
            .checked_add(14 * size_of::<usize>() as u64)?,
    )
}
impl ComputationFactAttribution {
    pub(in crate::domain_computation::primary_graph) fn retain_walk_memory(
        &mut self,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) {
        self.walk_memory = memory;
    }
    pub(in crate::domain_computation::primary_graph) fn structural_read_bytes(
        &self,
        key: &WorthQueryApplicationFactKey,
        read: ComputationRead,
    ) -> Option<u64> {
        let prior = self.readers.get(key);
        let mut bytes = if prior.is_none() {
            node::<(WorthQueryApplicationFactKey, ComputationFactReads)>(self.readers.len())?
        } else {
            0
        };
        if !self.calls.contains(&read) {
            bytes = bytes.checked_add(node::<ComputationRead>(self.calls.len())?)?;
        }
        if !self.fresh_calls.contains(&read) {
            bytes = bytes.checked_add(node::<ComputationRead>(self.fresh_calls.len())?)?;
        }
        let extra = match read {
            ComputationRead::Membership => 0,
            ComputationRead::ItemKey(id) if prior.is_none_or(|p| !p.item_keys.contains(&id)) => {
                node::<PartitionItemId>(prior.map_or(0, |p| p.item_keys.len()))?
            }
            ComputationRead::Partition(id) if prior.is_none_or(|p| !p.partitions.contains(&id)) => {
                node::<PartitionIdentity>(prior.map_or(0, |p| p.partitions.len()))?
            }
            _ => 0,
        };
        bytes.checked_add(extra)
    }
}
