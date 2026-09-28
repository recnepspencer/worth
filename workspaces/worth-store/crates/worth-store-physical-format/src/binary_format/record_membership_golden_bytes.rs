use crate::*;

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn generation(raw: u64) -> PhysicalGeneration {
    PhysicalGeneration::from_raw(raw).unwrap()
}

#[test]
fn membership_authority_artifacts_match_independent_golden_bytes() {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment_id = PhysicalSegmentId::from_raw(1).unwrap();
    let page_id = PhysicalPageId::from_raw(9).unwrap();
    let segment = authority
        .segment_cell(segment_id)
        .with_segment_generation(generation(3));
    let page = authority
        .page_cell(segment_id, page_id)
        .with_page_generation(generation(3));
    let data_segment = authority
        .segment_cell(segment_id)
        .with_segment_generation(generation(2));
    let page_entry = RecordSegmentPageManifestEntry::new(page, data_segment, 1, 0).unwrap();
    let segment_manifest =
        DurableSegmentManifest::new(format(), segment, 4, vec![page_entry]).unwrap();
    let mut segment_payload = vec![0_u8; 64];
    segment_payload[..8].copy_from_slice(&1_u64.to_le_bytes());
    segment_payload[8..12].copy_from_slice(&4_u32.to_le_bytes());
    segment_payload[12..16].copy_from_slice(&1_u32.to_le_bytes());
    segment_payload[24..32].copy_from_slice(&9_u64.to_le_bytes());
    segment_payload[32..40].copy_from_slice(&3_u64.to_le_bytes());
    segment_payload[40..48].copy_from_slice(&2_u64.to_le_bytes());
    segment_payload[48..52].copy_from_slice(&1_u32.to_le_bytes());
    assert_eq!(
        segment_manifest.encode(format()),
        independent_frame(5, 3, &segment_payload)
    );

    let record = PersistedRecordIdentity::new([0x22; 16], 7).unwrap();
    let extent = authority
        .record_extent_cell(PhysicalExtentId::from_raw(4).unwrap())
        .with_extent_generation(generation(5));
    let extent_manifest =
        DurableExtentManifest::new(format(), record, extent, 100, 16_384, 1, 4096).unwrap();
    let mut extent_payload = vec![0_u8; 56];
    extent_payload[..16].copy_from_slice(&[0x22; 16]);
    extent_payload[16..24].copy_from_slice(&7_u64.to_le_bytes());
    extent_payload[24..32].copy_from_slice(&4_u64.to_le_bytes());
    extent_payload[32..40].copy_from_slice(&100_u64.to_le_bytes());
    extent_payload[40..44].copy_from_slice(&16_384_u32.to_le_bytes());
    extent_payload[44..48].copy_from_slice(&1_u32.to_le_bytes());
    extent_payload[48..56].copy_from_slice(&4096_u64.to_le_bytes());
    assert_eq!(
        extent_manifest.encode(format()),
        independent_frame(6, 5, &extent_payload)
    );

    let entries = vec![
        RecordFreeSpaceManifestEntry::inline_frontier(7, 4, 2, 3).unwrap(),
        RecordFreeSpaceManifestEntry::arena_range(
            ExtentArenaRange::new(ExtentArenaId::new(5).unwrap(), 0, 8192).unwrap(),
            1,
        )
        .unwrap(),
    ];
    let free_block = PhysicalFreeSpaceMembershipBlock::leaf(8, 6, 1, entries, 2).unwrap();
    let mut free_block_payload = vec![0_u8; 120];
    free_block_payload[..8].copy_from_slice(&8_u64.to_le_bytes());
    free_block_payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
    free_block_payload[18..20].copy_from_slice(&2_u16.to_le_bytes());
    free_block_payload[20] = 1;
    free_block_payload[24..32].copy_from_slice(&6_u64.to_le_bytes());
    encode_free_entry(&mut free_block_payload[40..80], 1, 7, 4, 2, 3);
    encode_free_entry(&mut free_block_payload[80..120], 2, 5, 0, 8192, 1);
    let free_block_bytes = independent_frame(10, 1, &free_block_payload);
    assert_eq!(free_block.encode(format()), free_block_bytes);

    let block_checksum = independent_crc32c(&[&free_block_bytes]);
    let first = FreeSpaceKey::inline(7).unwrap();
    let last = FreeSpaceKey::arena(ExtentArenaId::new(5).unwrap(), 0);
    let free_reference =
        FreeSpaceBlockReference::new(6, 1, 0, block_checksum, first, last).unwrap();
    let free_header = DurableFreeSpaceManifestHeader::new(
        6,
        8,
        2,
        4,
        2,
        8,
        10,
        5,
        6,
        65536,
        4096,
        2,
        Some(free_reference),
    )
    .unwrap();
    let mut free_header_payload = vec![0_u8; 168];
    free_header_payload[..8].copy_from_slice(&6_u64.to_le_bytes());
    free_header_payload[8..16].copy_from_slice(&8_u64.to_le_bytes());
    free_header_payload[16..18].copy_from_slice(&2_u16.to_le_bytes());
    free_header_payload[18..22].copy_from_slice(&4_u32.to_le_bytes());
    free_header_payload[24..32].copy_from_slice(&2_u64.to_le_bytes());
    free_header_payload[32..40].copy_from_slice(&8_u64.to_le_bytes());
    free_header_payload[40..48].copy_from_slice(&10_u64.to_le_bytes());
    free_header_payload[48..56].copy_from_slice(&5_u64.to_le_bytes());
    free_header_payload[56..64].copy_from_slice(&2_u64.to_le_bytes());
    free_header_payload[64] = 1;
    encode_free_reference(&mut free_header_payload[72..144], free_reference);
    free_header_payload[144..152].copy_from_slice(&6_u64.to_le_bytes());
    free_header_payload[152..160].copy_from_slice(&65536_u64.to_le_bytes());
    free_header_payload[160..168].copy_from_slice(&4096_u64.to_le_bytes());
    let free_header_bytes = independent_frame(7, 6, &free_header_payload);
    assert_eq!(free_header.encode(format()), free_header_bytes);

    assert_eq!(
        DurableSegmentManifest::decode(&independent_frame(5, 3, &segment_payload), 4)
            .unwrap()
            .0,
        segment_manifest
    );
    assert_eq!(
        DurableExtentManifest::decode(&independent_frame(6, 5, &extent_payload))
            .unwrap()
            .0,
        extent_manifest
    );
    assert_eq!(
        PhysicalFreeSpaceMembershipBlock::decode(&free_block_bytes, 2)
            .unwrap()
            .0,
        free_block
    );
    assert_eq!(
        DurableFreeSpaceManifestHeader::decode(&free_header_bytes, 2)
            .unwrap()
            .0,
        free_header
    );
}

fn encode_free_entry(
    target: &mut [u8],
    class: u8,
    owner: u64,
    first: u64,
    count: u64,
    generation: u64,
) {
    target[0] = class;
    target[8..16].copy_from_slice(&owner.to_le_bytes());
    target[16..24].copy_from_slice(&first.to_le_bytes());
    target[24..32].copy_from_slice(&count.to_le_bytes());
    target[32..40].copy_from_slice(&generation.to_le_bytes());
}

fn encode_free_reference(target: &mut [u8], reference: FreeSpaceBlockReference) {
    target[..8].copy_from_slice(&reference.generation().to_le_bytes());
    target[8..16].copy_from_slice(&reference.block().to_le_bytes());
    target[16..18].copy_from_slice(&reference.level().to_le_bytes());
    target[20..24].copy_from_slice(&reference.checksum().to_le_bytes());
    encode_free_key(&mut target[24..48], reference.first());
    encode_free_key(&mut target[48..72], reference.last());
}

fn encode_free_key(target: &mut [u8], key: FreeSpaceKey) {
    target[0] = key.class() as u8;
    target[8..16].copy_from_slice(&key.owner().to_le_bytes());
    target[16..24].copy_from_slice(&key.offset().to_le_bytes());
}

const INDEPENDENT_FRAME_HEADER_BYTES: usize = 48;
const INDEPENDENT_CHECKSUM_OFFSET: usize = 44;

fn independent_frame(kind: u8, identity: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0_u8; INDEPENDENT_FRAME_HEADER_BYTES + payload.len()];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = kind;
    bytes[9] = 2;
    bytes[10..20].copy_from_slice(&[2, 0, 0, 64, 0, 0, 1, 1, 1, 24]);
    bytes[20..22].copy_from_slice(&(INDEPENDENT_FRAME_HEADER_BYTES as u16).to_le_bytes());
    bytes[24..28].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[28..36].copy_from_slice(&identity.to_le_bytes());
    bytes[INDEPENDENT_FRAME_HEADER_BYTES..].copy_from_slice(payload);
    let checksum = independent_crc32c(&[&bytes[..INDEPENDENT_CHECKSUM_OFFSET], payload]);
    bytes[INDEPENDENT_CHECKSUM_OFFSET..INDEPENDENT_FRAME_HEADER_BYTES]
        .copy_from_slice(&checksum.to_le_bytes());
    bytes
}

fn independent_crc32c(parts: &[&[u8]]) -> u32 {
    let mut crc = !0_u32;
    for part in parts {
        for byte in *part {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0x82f6_3b78 & 0_u32.wrapping_sub(crc & 1));
            }
        }
    }
    !crc
}
