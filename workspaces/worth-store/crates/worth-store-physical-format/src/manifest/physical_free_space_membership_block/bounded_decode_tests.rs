use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{DurableFrameKind, PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry};

use super::{
    BoundedFreeSpaceMembershipBlockDecodeDenial, FreeSpaceMembershipBlockDecodeLimits,
    PhysicalFreeSpaceMembershipBlock, BLOCK_PREFIX_BYTES, ENTRY_BYTES, REFERENCE_BYTES,
};
use crate::manifest::{FreeSpaceBlockReference, FreeSpaceKey};

#[test]
fn leaf_limit_wins_before_crossing_entry_decode() {
    let format = format();
    let block =
        PhysicalFreeSpaceMembershipBlock::leaf(7, 1, 1, vec![entry(1), entry(2)], 2).unwrap();
    let bytes = block.encode(format);
    let (_, frame) =
        decode_durable_frame(&bytes, DurableFrameKind::FreeSpaceMembershipBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[BLOCK_PREFIX_BYTES + ENTRY_BYTES + 8] = 0;
    let damaged = encode_durable_frame(
        DurableFrameKind::FreeSpaceMembershipBlock,
        format,
        1,
        &payload,
    );
    assert_eq!(
        PhysicalFreeSpaceMembershipBlock::decode_bounded(
            &damaged,
            2,
            FreeSpaceMembershipBlockDecodeLimits {
                leaf_entries: 1,
                branch_children: 0,
            },
        ),
        Err(BoundedFreeSpaceMembershipBlockDecodeDenial::LeafEntries {
            observed: 2,
            admitted: 1,
        })
    );
}

#[test]
fn branch_limit_wins_before_crossing_reference_decode() {
    let format = format();
    let block = PhysicalFreeSpaceMembershipBlock::branch(
        7,
        1,
        3,
        1,
        vec![reference(1, 1), reference(2, 2)],
        2,
    )
    .unwrap();
    let bytes = block.encode(format);
    let (_, frame) =
        decode_durable_frame(&bytes, DurableFrameKind::FreeSpaceMembershipBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[BLOCK_PREFIX_BYTES + REFERENCE_BYTES + 24] = 0;
    let damaged = encode_durable_frame(
        DurableFrameKind::FreeSpaceMembershipBlock,
        format,
        3,
        &payload,
    );
    assert_eq!(
        PhysicalFreeSpaceMembershipBlock::decode_bounded(
            &damaged,
            2,
            FreeSpaceMembershipBlockDecodeLimits {
                leaf_entries: 0,
                branch_children: 1,
            },
        ),
        Err(
            BoundedFreeSpaceMembershipBlockDecodeDenial::BranchChildren {
                observed: 2,
                admitted: 1,
            }
        )
    );
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

#[test]
fn reserved_free_frame_requires_full_backing_and_roundtrips() {
    let block = PhysicalFreeSpaceMembershipBlock::leaf(7, 1, 1, vec![entry(1)], 2).unwrap();
    let required = block.encoded_frame_bytes().unwrap();
    assert!(block
        .encode_in_reserved(format(), Vec::with_capacity(required - 1))
        .is_none());
    let bytes = block
        .encode_in_reserved(format(), Vec::with_capacity(required))
        .unwrap();
    assert_eq!(bytes.len(), required);
    assert_eq!(bytes[8], DurableFrameKind::FreeSpaceMembershipBlock as u8);
    assert_eq!(
        PhysicalFreeSpaceMembershipBlock::decode(&bytes, 2),
        Ok((block, format()))
    );
}

fn entry(owner: u64) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::inline_frontier(owner, 1, 1, 1).unwrap()
}

fn reference(owner: u64, block: u64) -> FreeSpaceBlockReference {
    let key = FreeSpaceKey::inline(owner).unwrap();
    FreeSpaceBlockReference::new(1, block, 0, 1, key, key).unwrap()
}
