//! Storage of the recorded calls that travel with a full preparation.
use super::*;
impl<Key: Send + Sync + 'static, Item: ChargedBytes> FullRecording<Key, Item> {
    pub(in super::super::super) fn transport_bytes(&self) -> u64 {
        let Self {
            basis: _,
            membership,
            item_keys,
            routing: _,
            partitions,
            measured: _,
        } = self;
        let mut bytes = 0_u64;
        if let Some((call, items, digests)) = membership {
            bytes = bytes.saturating_add(call.charge.additional_bytes().unwrap_or(u64::MAX));
            for item in items.values() {
                bytes = bytes
                    .saturating_add(size_of::<(PartitionItemId, Item)>() as u64)
                    .saturating_add(item.additional_charged_bytes());
            }
            bytes = bytes
                .saturating_add((digests.len() * size_of::<(PartitionItemId, [u8; 32])>()) as u64);
        }
        for call in item_keys.values() {
            bytes = bytes
                .saturating_add(size_of::<(PartitionItemId, RetainedCall)>() as u64)
                .saturating_add(call.charge.additional_bytes().unwrap_or(u64::MAX));
        }
        for (_, key_bytes, charge) in partitions.values() {
            bytes = bytes
                .saturating_add(
                    size_of::<(PartitionIdentity, Arc<Key>, u64, ComputationCallCharge)>() as u64,
                )
                .saturating_add(*key_bytes)
                .saturating_add(charge.additional_bytes().unwrap_or(u64::MAX));
        }
        // Routing has its own request reservation, retained in this recording.
        bytes
    }
}
