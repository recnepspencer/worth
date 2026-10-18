//! Transport keeps existing gathered and prior custody, and charges new metadata.
use super::*;
use worth_execution::ChargedBytes;
impl<Key, Item: ChargedBytes, Reduced, Gathered: ChargedBytes>
    PreparedIncremental<Key, Item, Reduced, Gathered>
{
    pub(in super::super::super) fn transport_bytes(&self) -> u64 {
        let Self {
            validated_at: _,
            remaining_work: _,
            declared_bytes: _,
            partitions,
            removed,
            next,
            basis: _,
            prior,
        } = self;
        let mut bytes =
            (partitions.len() * size_of::<(PartitionIdentity, Recomputed<Key, Gathered>)>()) as u64;
        bytes = bytes.saturating_add(std::mem::size_of_val(removed.as_slice()) as u64);
        for Recomputed {
            value,
            key: _,
            key_bytes,
            gather,
            memory,
        } in partitions.values()
        {
            bytes = bytes
                .saturating_add(
                    value
                        .additional_charged_bytes()
                        .saturating_sub(memory.held_bytes()),
                )
                .saturating_add(*key_bytes)
                .saturating_add(
                    gather
                        .as_ref()
                        .map_or(0, |call| call.additional_bytes().unwrap_or(u64::MAX)),
                );
        }
        let NextPartitioning {
            items,
            digests,
            membership,
            item_keys,
            routing: _,
            routing_memory: _,
            carried_membership,
            carried_items,
        } = next;
        let retained = prior.typed();
        if !Arc::ptr_eq(items, &retained.items) {
            for item in items.values() {
                bytes = bytes
                    .saturating_add(size_of::<(PartitionItemId, Item)>() as u64)
                    .saturating_add(item.additional_charged_bytes());
            }
        }
        if !Arc::ptr_eq(digests, &retained.digests) {
            bytes = bytes
                .saturating_add((digests.len() * size_of::<(PartitionItemId, [u8; 32])>()) as u64);
        }
        if !carried_membership {
            bytes = bytes.saturating_add(
                membership
                    .as_ref()
                    .map_or(0, |call| call.charge.additional_bytes().unwrap_or(u64::MAX)),
            );
        }
        for (item, call) in item_keys {
            bytes = bytes.saturating_add(size_of::<(PartitionItemId, RetainedCall)>() as u64);
            if !carried_items.contains(item) {
                bytes = bytes.saturating_add(call.charge.additional_bytes().unwrap_or(u64::MAX));
            }
        }
        bytes.saturating_add((carried_items.len() * size_of::<PartitionItemId>()) as u64)
    }
    pub(in super::super::super) fn partition_count(&self) -> usize {
        self.next.routing.partitions().len()
    }
}
