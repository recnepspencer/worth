use crate::store_namespace::{ProposedStoreIdentity, StableStoreIdentity};
use crate::*;
use sha2::{Digest, Sha256};

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn store(byte: u8) -> StableStoreIdentity {
    StableStoreIdentity::from_published_record(
        ProposedStoreIdentity::from_nonzero_bytes([byte; 16]).unwrap(),
    )
}

fn root_entry(generation: u64) -> CurrentRootCatalogEntry {
    CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(generation).unwrap())
}

fn generation(raw: u64) -> PhysicalGeneration {
    PhysicalGeneration::from_raw(raw).unwrap()
}

fn segment_cell(segment: u64, generation_raw: u64) -> SegmentGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .segment_cell(PhysicalSegmentId::from_raw(segment).unwrap())
        .with_segment_generation(generation(generation_raw))
}

fn page_cell(page: u64, generation_raw: u64) -> PageGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(page).unwrap(),
        )
        .with_page_generation(generation(generation_raw))
}

fn slot_cell(page: u64, slot: u16, generation_raw: u64) -> SlotGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(page).unwrap(),
            PhysicalRecordSlot::from_raw(slot).unwrap(),
        )
        .with_slot_generation(generation(generation_raw))
}

fn extent_cell(extent: u64, generation_raw: u64) -> RecordExtentGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent).unwrap())
        .with_extent_generation(generation(generation_raw))
}

fn inline_placement(
    record: PersistedRecordIdentity,
    generation: u64,
    page: u64,
    slot: u16,
    payload_bytes: u64,
) -> CurrentPhysicalRecordPlacement {
    CurrentPhysicalRecordPlacement::Inline(
        DurableInlineRecordPlacement::new(
            record,
            PhysicalGenerationAuthority::for_canonical_physical_format()
                .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
                .with_segment_generation(self::generation(generation)),
            page_cell(page, generation),
            slot_cell(page, slot.saturating_add(1), generation),
            1024,
            payload_bytes,
        )
        .unwrap(),
    )
}

#[test]
fn crc32c_matches_the_castagnoli_reference_vector() {
    assert_eq!(
        crate::record_framing::crc32c::checksum(&[b"123456789"]),
        0xe306_9283
    );
}

#[test]
fn current_catalog_golden_bytes_are_bit_exact() {
    let store = store(0x11);
    let bytes = BootstrapCatalog::new(store, format(), root_entry(7)).encode();
    assert_eq!(&bytes[..10], b"WRC5FRM\0\x01\x02");
    assert_eq!(&bytes[10..20], &[2, 0, 0, 64, 0, 0, 1, 1, 1, 24]);
    assert_eq!(&bytes[20..28], &[48, 0, 0, 0, 34, 0, 0, 0]);
    assert_eq!(&bytes[28..36], &7_u64.to_le_bytes());
    assert_eq!(&bytes[36..44], &0_u64.to_le_bytes());
    assert_eq!(&bytes[48..64], &[0x11; 16]);
    assert_eq!(&bytes[64..72], &7_u64.to_le_bytes());
    assert_eq!(
        BootstrapCatalog::decode(&bytes)
            .unwrap()
            .current_root()
            .generation()
            .get(),
        7
    );
    assert_eq!(
        digest(&bytes),
        "1881f9c5e23452027d8626e9584f582f371639e5f5f38c1e651a1c3b51ff3174"
    );
}

#[test]
fn manifest_and_empty_payload_page_round_trip_independently() {
    let record = PersistedRecordIdentity::new([9; 16], 1).unwrap();
    let placement = inline_placement(record, 2, 1, 0, 0);
    let block = PhysicalRootRoutingBlock::leaf(7, 2, 1, vec![placement], 4).unwrap();
    let encoded_block = block.encode(format());
    let routing_reference = block.reference(durable_artifact_checksum(&encoded_block));
    let free_key = FreeSpaceKey::arena(ExtentArenaId::new(1).unwrap(), 0);
    let free_reference =
        FreeSpaceBlockReference::new(2, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(2, 7, 4, 0x8a9b_acbd)
        .record_count(1)
        .next_block(2)
        .routing_root(Some(routing_reference))
        .free_space_root(Some(free_reference))
        .last_inline_record(Some(record))
        .last_inline_segment(Some(segment_cell(1, 1)))
        .admit()
        .unwrap();
    let encoded_manifest = manifest.encode(format());
    let (decoded, decoded_format) =
        DurablePhysicalRootManifest::decode(&encoded_manifest, 4).unwrap();
    assert_eq!(decoded_format, format());
    assert_eq!(decoded.routing_root(), Some(routing_reference));
    let (decoded_block, block_format) =
        PhysicalRootRoutingBlock::decode(&encoded_block, 4).unwrap();
    assert_eq!(block_format, format());
    assert_eq!(decoded_block.entries().unwrap()[0].record(), record);
    assert_eq!(
        digest(&encoded_manifest),
        "1f8703e63ad822d255222d2ffc5f3e9a01d7707c4df994a6a9e09d8883c7e0e4"
    );
    assert_eq!(
        digest(&encoded_block),
        "927d68d03dff0546e06c63583cd1ddab9239a38fbd1a2022d65461586ec43478"
    );

    let page_cell = page_cell(1, 2);
    let slot_cell = slot_cell(1, 1, 2);
    let page = encode_inline_page(
        format(),
        page_cell,
        &[InlineRecordAppend::new(record, slot_cell, &[])],
    )
    .unwrap();
    assert_eq!(
        digest(&page),
        "2d5106a0cde679cc4e8135022f24d9eee00ae57b7071be12215afda797026b1b"
    );
    assert_eq!(page.len(), format().page_bytes() as usize);
    let range = decode_inline_record(&page, record, page_cell, slot_cell)
        .unwrap()
        .0
        .range();
    assert!(page[range].is_empty());
    let empty_page = encode_inline_page(format(), self::page_cell(2, 3), &[]).unwrap();
    assert_eq!(
        digest(&empty_page),
        "ae8aef05eac1dd3e93dd418ca8be3abcfbd20a103e68e073a1f2d4f0e07fae2c"
    );
    assert_eq!(&empty_page[64..66], &[0, 0]);

    let coordinate = ExtentChunkCoordinate::new(record, extent_cell(1, 4), 12, 0, 1).unwrap();
    let extent = encode_extent_chunk(format(), coordinate, b"large-record").unwrap();
    assert_eq!(
        decode_extent_chunk(&extent, coordinate).unwrap().0,
        b"large-record"
    );
}

#[test]
fn full_slot_directory_golden_bytes_are_bit_exact() {
    let records = (1..=407_u64)
        .map(|ordinal| PersistedRecordIdentity::new([5; 16], ordinal).unwrap())
        .collect::<Vec<_>>();
    let page_cell = page_cell(1, 9);
    let appends = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            InlineRecordAppend::new(*record, slot_cell(1, index as u16 + 1, 9), &[])
        })
        .collect::<Vec<_>>();
    let page = encode_inline_page(format(), page_cell, &appends).unwrap();
    assert_eq!(
        digest(&page),
        "938cf88b1bc14f059135ed8d27dee39ee5cf56109199260ffa2a9f31ffba6ffd"
    );
    assert!(
        page[decode_inline_record(&page, records[0], page_cell, slot_cell(1, 1, 9))
            .unwrap()
            .0
            .range()]
        .is_empty()
    );
    assert!(
        page[decode_inline_record(&page, records[406], page_cell, slot_cell(1, 407, 9))
            .unwrap()
            .0
            .range()]
        .is_empty()
    );
    assert_eq!(
        encode_inline_page(
            format(),
            page_cell,
            &appends
                .into_iter()
                .chain(std::iter::once(InlineRecordAppend::new(
                    PersistedRecordIdentity::new([5; 16], 408).unwrap(),
                    slot_cell(1, 408, 9),
                    &[],
                )))
                .collect::<Vec<_>>()
        ),
        Err(InlinePageDenial::PageFull)
    );
}

#[test]
fn boundary_field_widths_round_trip_without_narrowing() {
    let record = PersistedRecordIdentity::new([0x66; 16], u64::MAX).unwrap();
    let placement = inline_placement(
        record,
        u64::MAX,
        u64::from(u32::MAX),
        u16::MAX,
        u64::from(u32::MAX),
    );
    let block =
        PhysicalRootRoutingBlock::leaf(u64::MAX, u64::MAX, u64::MAX, vec![placement], 2).unwrap();
    let bytes = block.encode(format());
    let decoded = PhysicalRootRoutingBlock::decode(&bytes, 2).unwrap().0;
    assert_eq!(decoded.block(), u64::MAX);
    assert_eq!(decoded.tree_identity(), u64::MAX);
    assert_eq!(decoded.entries().unwrap(), &[placement]);
}

#[test]
fn root_manifest_rejects_duplicate_physical_slot_ownership() {
    let first = inline_placement(
        PersistedRecordIdentity::new([0x41; 16], 1).unwrap(),
        7,
        2,
        3,
        4,
    );
    let second = inline_placement(
        PersistedRecordIdentity::new([0x42; 16], 1).unwrap(),
        7,
        2,
        3,
        4,
    );

    assert!(PhysicalRootRoutingBlock::leaf(8, 1, 1, vec![first, second], 2).is_none());
}

#[test]
fn root_manifest_requires_canonical_record_identity_order() {
    let later = inline_placement(
        PersistedRecordIdentity::new([0x42; 16], 1).unwrap(),
        7,
        2,
        3,
        4,
    );
    let earlier = inline_placement(
        PersistedRecordIdentity::new([0x41; 16], 1).unwrap(),
        7,
        3,
        3,
        4,
    );
    assert!(PhysicalRootRoutingBlock::leaf(8, 1, 1, vec![later, earlier], 2).is_none());
}

#[test]
fn root_branch_rejects_overlap_disorder_and_future_generation() {
    let first = PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    let middle = PersistedRecordIdentity::new([0x42; 16], 1).unwrap();
    let last = PersistedRecordIdentity::new([0x43; 16], 1).unwrap();
    let left = ManifestBlockReference::new(1, 1, 0, 11, first, middle).unwrap();
    let overlap = ManifestBlockReference::new(1, 2, 0, 12, middle, last).unwrap();
    let right = ManifestBlockReference::new(1, 2, 0, 12, last, last).unwrap();
    let future = ManifestBlockReference::new(3, 2, 0, 12, last, last).unwrap();

    assert!(PhysicalRootRoutingBlock::branch(8, 2, 3, 1, vec![left, overlap], 2).is_none());
    assert!(PhysicalRootRoutingBlock::branch(8, 2, 3, 1, vec![right, left], 2).is_none());
    assert!(PhysicalRootRoutingBlock::branch(8, 2, 3, 1, vec![left, future], 2).is_none());
}

#[test]
fn zero_crc_is_valid_reference_data_not_an_absence_sentinel() {
    let record = PersistedRecordIdentity::new([0x41; 16], 1).unwrap();
    assert!(ManifestBlockReference::new(1, 1, 0, 0, record, record).is_some());

    let segment_key = SegmentPageKey::new(
        PhysicalSegmentId::from_raw(1).unwrap(),
        PhysicalPageId::from_raw(1).unwrap(),
    );
    assert!(SegmentManifestBlockReference::new(1, 1, 0, 0, segment_key, segment_key).is_some());

    let free_key = FreeSpaceKey::inline(1).unwrap();
    assert!(FreeSpaceBlockReference::new(1, 1, 0, 0, free_key, free_key).is_some());
}

#[test]
fn extent_coordinate_rejects_an_invalid_generation() {
    let record = PersistedRecordIdentity::new([7; 16], 1).unwrap();
    assert!(ExtentChunkCoordinate::new(record, extent_cell(1, 7), 0, 0, 1).is_none());
}

#[test]
fn record_placements_reject_lengths_outside_the_c5_record_width() {
    let record = PersistedRecordIdentity::new([7; 16], 1).unwrap();
    assert!(DurableInlineRecordPlacement::new(
        record,
        segment_cell(1, 1),
        page_cell(1, 1),
        slot_cell(1, 1, 1),
        1,
        u64::from(u32::MAX) + 1,
    )
    .is_none());
    assert!(DurableExtentRecordPlacement::new(
        record,
        extent_cell(1, 1),
        u64::from(u32::MAX) + 1,
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), 0, 20480).unwrap()
    )
    .is_none());
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn future_format_is_rejected_before_payload_or_checksum_work() {
    let store = store(0x22);
    let mut bytes = BootstrapCatalog::new(store, format(), root_entry(1)).encode();
    bytes[10..12].copy_from_slice(&3_u16.to_le_bytes());
    assert!(matches!(
        BootstrapCatalog::decode(&bytes),
        Err(BootstrapCatalogDenial::Frame(
            DurableFrameDenial::UnsupportedFormat(PhysicalRecordFormatDenial::UnsupportedVersion(
                3
            ))
        ))
    ));
}

#[test]
fn checksum_covers_identity_header_and_full_payload() {
    let store = store(0x33);
    let bytes = BootstrapCatalog::new(store, format(), root_entry(1)).encode();
    for offset in [28, 40, BOOTSTRAP_CATALOG_BYTES - 1] {
        let mut corrupt = bytes;
        corrupt[offset] ^= 1;
        assert!(
            matches!(
                BootstrapCatalog::decode(&corrupt),
                Err(BootstrapCatalogDenial::Frame(
                    DurableFrameDenial::IntegrityMismatch
                ))
            ),
            "C5_PREDICATE:minimum-integrity"
        );
    }
}
