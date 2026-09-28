use crate::store_namespace::{ProposedStoreIdentity, StableStoreIdentity};
use crate::*;

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

#[test]
fn independent_decode_fixtures_recover_every_phase_one_artifact() {
    let record = PersistedRecordIdentity::new([0x44; 16], 9).unwrap();

    let mut catalog_payload = vec![0_u8; 34];
    catalog_payload[..16].copy_from_slice(&[0x55; 16]);
    catalog_payload[16..24].copy_from_slice(&5_u64.to_le_bytes());
    catalog_payload[24..34].copy_from_slice(&canonical_format_bytes());
    let catalog_bytes = independent_frame(1, 5, &catalog_payload);
    let catalog = BootstrapCatalog::decode(&catalog_bytes).unwrap();
    assert_eq!(catalog.store_identity(), store(0x55));
    assert_eq!(catalog.current_root(), root_entry(5));

    let mut block_payload = vec![0_u8; 128];
    block_payload[..8].copy_from_slice(&9_u64.to_le_bytes());
    block_payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
    block_payload[18..20].copy_from_slice(&1_u16.to_le_bytes());
    block_payload[20] = 1;
    block_payload[24..32].copy_from_slice(&5_u64.to_le_bytes());
    block_payload[40..56].copy_from_slice(&record.allocation_epoch());
    block_payload[56..64].copy_from_slice(&record.ordinal().to_le_bytes());
    block_payload[64] = 1;
    block_payload[72..80].copy_from_slice(&1_u64.to_le_bytes());
    block_payload[80..88].copy_from_slice(&1_u64.to_le_bytes());
    block_payload[88..96].copy_from_slice(&6_u64.to_le_bytes());
    block_payload[96..104].copy_from_slice(&6_u64.to_le_bytes());
    block_payload[104..112].copy_from_slice(&1_u64.to_le_bytes());
    block_payload[112..120].copy_from_slice(&3_u64.to_le_bytes());
    block_payload[120..124].copy_from_slice(&1024_u32.to_le_bytes());
    block_payload[124..126].copy_from_slice(&1_u16.to_le_bytes());
    let block_bytes = independent_frame(8, 1, &block_payload);
    let block_checksum = durable_artifact_checksum(&block_bytes);

    let mut manifest_payload = vec![0_u8; 336];
    manifest_payload[..8].copy_from_slice(&5_u64.to_le_bytes());
    manifest_payload[8..16].copy_from_slice(&9_u64.to_le_bytes());
    manifest_payload[16..18].copy_from_slice(&2_u16.to_le_bytes());
    manifest_payload[24..32].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[32..40].copy_from_slice(&2_u64.to_le_bytes());
    manifest_payload[40] = 1;
    manifest_payload[48..56].copy_from_slice(&5_u64.to_le_bytes());
    manifest_payload[56..64].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[68..72].copy_from_slice(&block_checksum.to_le_bytes());
    manifest_payload[72..88].copy_from_slice(&record.allocation_epoch());
    manifest_payload[88..96].copy_from_slice(&record.ordinal().to_le_bytes());
    manifest_payload[96..112].copy_from_slice(&record.allocation_epoch());
    manifest_payload[112..120].copy_from_slice(&record.ordinal().to_le_bytes());
    manifest_payload[120] = 1;
    manifest_payload[128..144].copy_from_slice(&record.allocation_epoch());
    manifest_payload[144..152].copy_from_slice(&record.ordinal().to_le_bytes());
    manifest_payload[152..156].copy_from_slice(&0x8a9b_acbd_u32.to_le_bytes());
    manifest_payload[224..232].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[232] = 1;
    manifest_payload[240..248].copy_from_slice(&5_u64.to_le_bytes());
    manifest_payload[248..256].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[260..264].copy_from_slice(&0x0102_0304_u32.to_le_bytes());
    manifest_payload[264] = 2;
    manifest_payload[272..280].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[288] = 2;
    manifest_payload[296..304].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[312] = 1;
    manifest_payload[320..328].copy_from_slice(&1_u64.to_le_bytes());
    manifest_payload[328..336].copy_from_slice(&1_u64.to_le_bytes());
    let manifest_bytes = independent_frame(2, 5, &manifest_payload);
    let manifest = DurablePhysicalRootManifest::decode(&manifest_bytes, 2)
        .unwrap()
        .0;

    let block = PhysicalRootRoutingBlock::decode(&block_bytes, 2).unwrap().0;
    assert_eq!(
        manifest.routing_root(),
        Some(block.reference(block_checksum))
    );
    assert_eq!(block.entries().unwrap()[0].payload_bytes(), 3);

    let page_payload_bytes = format().page_bytes() as usize - INDEPENDENT_FRAME_HEADER_BYTES;
    let mut page_payload = vec![0_u8; page_payload_bytes];
    page_payload[..8].copy_from_slice(&1_u64.to_le_bytes());
    page_payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
    page_payload[16..18].copy_from_slice(&1_u16.to_le_bytes());
    page_payload[24..40].copy_from_slice(&record.allocation_epoch());
    page_payload[40..48].copy_from_slice(&record.ordinal().to_le_bytes());
    let record_offset = page_payload_bytes - 3;
    page_payload[48..52].copy_from_slice(&(record_offset as u32).to_le_bytes());
    page_payload[52..56].copy_from_slice(&3_u32.to_le_bytes());
    page_payload[56..64].copy_from_slice(&1_u64.to_le_bytes());
    page_payload[record_offset..].copy_from_slice(b"raw");
    let page_bytes = independent_frame(3, 6, &page_payload);
    let range = decode_inline_record(&page_bytes, record, page_cell(1, 6), slot_cell(1, 1, 1))
        .unwrap()
        .0
        .range();
    assert_eq!(&page_bytes[range], b"raw");

    let extent_id = PhysicalExtentId::from_raw(2).unwrap();
    let mut extent_payload = vec![0_u8; 67];
    extent_payload[..16].copy_from_slice(&record.allocation_epoch());
    extent_payload[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
    extent_payload[24..32].copy_from_slice(&extent_id.get().to_le_bytes());
    extent_payload[32..40].copy_from_slice(&7_u64.to_le_bytes());
    extent_payload[40..48].copy_from_slice(&3_u64.to_le_bytes());
    extent_payload[56..60].copy_from_slice(&3_u32.to_le_bytes());
    extent_payload[64..].copy_from_slice(b"raw");
    let extent_bytes = independent_frame(4, 1, &extent_payload);
    assert_eq!(
        decode_extent_chunk(
            &extent_bytes,
            ExtentChunkCoordinate::new(record, extent_cell(2, 7), 3, 0, 1).unwrap()
        )
        .unwrap()
        .0,
        b"raw"
    );
}

#[test]
fn inline_page_rejects_checksummed_noncanonical_gap_bytes() {
    let record = PersistedRecordIdentity::new([0x31; 16], 1).unwrap();
    let payload_bytes = format().page_bytes() as usize - INDEPENDENT_FRAME_HEADER_BYTES;
    let mut payload = vec![0_u8; payload_bytes];
    payload[..8].copy_from_slice(&1_u64.to_le_bytes());
    payload[8..16].copy_from_slice(&1_u64.to_le_bytes());
    payload[16..18].copy_from_slice(&1_u16.to_le_bytes());
    payload[24..40].copy_from_slice(&record.allocation_epoch());
    payload[40..48].copy_from_slice(&record.ordinal().to_le_bytes());
    let record_offset = payload_bytes - 3;
    payload[48..52].copy_from_slice(&(record_offset as u32).to_le_bytes());
    payload[52..56].copy_from_slice(&3_u32.to_le_bytes());
    payload[56..64].copy_from_slice(&1_u64.to_le_bytes());
    payload[record_offset - 1] = 0xa5;
    payload[record_offset..].copy_from_slice(b"raw");

    assert_eq!(
        decode_inline_record(
            &independent_frame(3, 1, &payload),
            record,
            page_cell(1, 1),
            slot_cell(1, 1, 1),
        ),
        Err(InlinePageDenial::ReservedFieldNonZero)
    );
}

fn canonical_format_bytes() -> [u8; 10] {
    [2, 0, 0, 64, 0, 0, 1, 1, 1, 24]
}

const INDEPENDENT_FRAME_HEADER_BYTES: usize = 48;
const INDEPENDENT_CHECKSUM_OFFSET: usize = 44;

fn independent_frame(kind: u8, identity: u64, payload: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0_u8; INDEPENDENT_FRAME_HEADER_BYTES + payload.len()];
    bytes[..8].copy_from_slice(b"WRC5FRM\0");
    bytes[8] = kind;
    bytes[9] = 2;
    bytes[10..20].copy_from_slice(&canonical_format_bytes());
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
