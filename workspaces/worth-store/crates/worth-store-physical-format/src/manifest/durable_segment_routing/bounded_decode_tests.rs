use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    DurableFrameKind, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordFormatDeclaration, PhysicalSegmentId, RecordSegmentPageManifestEntry,
};

use super::{
    BoundedSegmentMembershipBlockDecodeDenial, PhysicalSegmentMembershipBlock,
    SegmentManifestBlockReference, SegmentMembershipBlockDecodeLimits, SegmentPageKey,
    BLOCK_PREFIX_BYTES, LEAF_ENTRY_BYTES, REFERENCE_BYTES,
};

#[test]
fn leaf_limit_wins_before_crossing_entry_decode() {
    let format = format();
    let block = PhysicalSegmentMembershipBlock::leaf(7, 1, 1, vec![entry(1), entry(2)], 2).unwrap();
    let bytes = block.encode(format);
    let (_, frame) =
        decode_durable_frame(&bytes, DurableFrameKind::SegmentMembershipBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[BLOCK_PREFIX_BYTES + LEAF_ENTRY_BYTES + 8] = 0;
    let damaged = encode_durable_frame(
        DurableFrameKind::SegmentMembershipBlock,
        format,
        1,
        &payload,
    );
    assert_eq!(
        PhysicalSegmentMembershipBlock::decode_bounded(
            &damaged,
            2,
            SegmentMembershipBlockDecodeLimits {
                leaf_entries: 1,
                branch_children: 0,
            },
        ),
        Err(BoundedSegmentMembershipBlockDecodeDenial::LeafEntries {
            observed: 2,
            admitted: 1,
        })
    );
}

#[test]
fn branch_limit_wins_before_crossing_reference_decode() {
    let format = format();
    let children = vec![reference(1, 1), reference(2, 2)];
    let block = PhysicalSegmentMembershipBlock::branch(7, 1, 3, 1, children, 2).unwrap();
    let bytes = block.encode(format);
    let (_, frame) =
        decode_durable_frame(&bytes, DurableFrameKind::SegmentMembershipBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[BLOCK_PREFIX_BYTES + REFERENCE_BYTES + 24] = 0;
    let damaged = encode_durable_frame(
        DurableFrameKind::SegmentMembershipBlock,
        format,
        3,
        &payload,
    );
    assert_eq!(
        PhysicalSegmentMembershipBlock::decode_bounded(
            &damaged,
            2,
            SegmentMembershipBlockDecodeLimits {
                leaf_entries: 0,
                branch_children: 1,
            },
        ),
        Err(BoundedSegmentMembershipBlockDecodeDenial::BranchChildren {
            observed: 2,
            admitted: 1,
        })
    );
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

#[test]
fn reserved_segment_frame_requires_full_backing_and_roundtrips() {
    let block = PhysicalSegmentMembershipBlock::leaf(7, 1, 1, vec![entry(1)], 2).unwrap();
    let required = block.encoded_frame_bytes().unwrap();
    assert!(block
        .encode_in_reserved(format(), Vec::with_capacity(required - 1))
        .is_none());
    let bytes = block
        .encode_in_reserved(format(), Vec::with_capacity(required))
        .unwrap();
    assert_eq!(bytes.len(), required);
    assert_eq!(bytes[8], DurableFrameKind::SegmentMembershipBlock as u8);
    assert_eq!(
        PhysicalSegmentMembershipBlock::decode(&bytes, 2),
        Ok((block, format()))
    );
}

fn entry(page: u64) -> RecordSegmentPageManifestEntry {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, PhysicalPageId::from_raw(page).unwrap())
            .with_page_generation(generation),
        authority
            .segment_cell(segment)
            .with_segment_generation(generation),
        2,
        (page - 1) as u32,
    )
    .unwrap()
}

fn reference(page: u64, block: u64) -> SegmentManifestBlockReference {
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let key = SegmentPageKey::new(segment, PhysicalPageId::from_raw(page).unwrap());
    SegmentManifestBlockReference::new(1, block, 0, 1, key, key).unwrap()
}
