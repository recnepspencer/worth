use crate::manifest::release_custody_head::block::ReleaseCustodyHeadBlockV1;
use crate::manifest::release_custody_head::entry::ENTRY_BYTES;
use crate::manifest::release_custody_head::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};
use crate::record_framing::DURABLE_FRAME_HEADER_BYTES;
use crate::PhysicalRecordFormatDeclaration;

use super::{ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1};

const NODE_PREFIX_BYTES: usize = 40;

fn array_bytes<T>(count: usize) -> Option<u64> {
    u64::try_from(count)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

/// Structural upper bound for the extra heap concurrently owned by exact COW
/// verification. The persisted effect's existing buffers are not counted here.
pub(super) fn additional_peak_bytes(
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    claimed_new_blocks: usize,
    format: PhysicalRecordFormatDeclaration,
) -> Option<u64> {
    u16::try_from(source_path.len().max(1)).ok()?;
    let new_blocks = u16::try_from(claimed_new_blocks).ok()?;
    if new_blocks == 0 {
        return None;
    }
    let frame_bytes = u64::from(format.page_size().bytes());
    let payload_capacity = usize::try_from(frame_bytes)
        .ok()?
        .checked_sub(DURABLE_FRAME_HEADER_BYTES + NODE_PREFIX_BYTES)?;
    let leaf_entries = payload_capacity / ENTRY_BYTES;
    let branch_children = payload_capacity / ReleaseCustodyHeadBlockReferenceV1::ENCODED_BYTES;
    if leaf_entries == 0 || branch_children == 0 {
        return None;
    }

    let decoded_nodes = source_path.iter().try_fold(
        array_bytes::<ReleaseCustodyHeadBlockV1>(source_path.len())?,
        |sum, node| {
            let entries = if node.reference().level() == 0 {
                array_bytes::<ReleaseCustodyHeadEntryV1>(leaf_entries)?
            } else {
                array_bytes::<ReleaseCustodyHeadBlockReferenceV1>(branch_children)?
            };
            sum.checked_add(entries)
        },
    )?;
    let writer = array_bytes::<ReleaseCustodyHeadNodeWriteV1>(claimed_new_blocks)?
        .checked_add(frame_bytes.checked_mul(u64::from(new_blocks))?)?;
    let leaf_cow = array_bytes::<ReleaseCustodyHeadEntryV1>(leaf_entries.checked_add(1)?)?;
    let branch_cow =
        array_bytes::<ReleaseCustodyHeadBlockReferenceV1>(branch_children.checked_add(1)?)?;
    let cow_vectors = leaf_cow.max(branch_cow).checked_mul(3)?;
    let replacement = array_bytes::<ReleaseCustodyHeadBlockReferenceV1>(4)?;
    let encode_buffers = frame_bytes.checked_mul(2)?;
    decoded_nodes
        .checked_add(writer)?
        .checked_add(cow_vectors)?
        .checked_add(replacement)?
        .checked_add(encode_buffers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_array_charge_rejects_overflow() {
        assert_eq!(array_bytes::<ReleaseCustodyHeadBlockV1>(usize::MAX), None);
    }

    #[test]
    fn one_write_still_charges_cow_and_encode_buffers() {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let bytes = additional_peak_bytes(&[], 1, format).unwrap();
        assert!(bytes > u64::from(format.page_size().bytes()));
        assert_eq!(additional_peak_bytes(&[], 0, format), None);
    }
}
