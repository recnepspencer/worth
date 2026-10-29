pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::time::Duration};

pub(crate) use {worth_store::physical_runtime::BlobCheckpointLimit, worth_store::physical_runtime::BlobIngestDeclaration, worth_store::physical_runtime::BlobReadFailure, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::LayoutRebuildFailure, worth_store::physical_runtime::LayoutRebuildLimits, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest, worth_store::physical_runtime::PhysicalIndexPointKey, worth_store::physical_runtime::PhysicalIntegrityScrubDeferral, worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalWorkCounterStage, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkPressureClass, worth_store::physical_runtime::PublishedBlobGeneration, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadWorkDenial};
pub(crate) use {worth_store_blob_chunks::BlobChunkSize};
pub(crate) use {worth_store_contracts::DurableArtifactFamilyId};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot};
pub(crate) use {worth_store_physical_format::BlobRecordDenial};
pub(crate) use {worth_store_physical_integrity::PhysicalIntegrityObservationOutcome, worth_store_physical_integrity::PhysicalIntegrityRejection};

pub(crate) use {crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};



pub(crate) const CHUNK_BYTES: usize = 64 * 1024;
pub(crate) const C5_HEADER: usize = 48;
pub(crate) const C5_EXTENT_METADATA: usize = 64;
pub(crate) const C11_HEADER: usize = 48;
pub(crate) const C11_CHUNK_CLAIM: usize = 80;
pub(crate) const C11_CONTENT_PREFIX: usize = 12;

pub(crate) fn hex_record(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
    let mut encoded = [0_u8; 24];
    encoded[..16].copy_from_slice(&record.allocation_epoch());
    encoded[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    encoded.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn read_one_byte(
    blobs: &worth_store::physical_runtime::PhysicalBlobFacade<'_>,
    published: PublishedBlobGeneration,
    scope: &worth_store::physical_runtime::AdmittedBlobScope,
    offset: u64,
    limits: BlobReadLimits,
) -> u8 {
    let mut read = blobs.read(published, scope, offset, 1, limits).unwrap();
    let mut byte = [0_u8; 1];
    assert_eq!(read.read_next(&mut byte).unwrap(), 1);
    byte[0]
}

pub(crate) fn flip_middle_chunk_payload_reseal_outer(root: &Path) {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(root.join("families/records/arenas")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "data") {
            continue;
        }
        let media = std::fs::read(&path).unwrap();
        for inner_offset in 0..media.len().saturating_sub(C11_HEADER) {
            let Some(outer_offset) = inner_offset.checked_sub(C5_HEADER + C5_EXTENT_METADATA)
            else {
                continue;
            };
            if !media[inner_offset..].starts_with(b"WRC11BLB")
                || media[inner_offset + 8] != 2
                || !media[outer_offset..].starts_with(b"WRC5FRM\0")
                || media[outer_offset + 8] != 4
            {
                continue;
            }
            let ordinal = u64::from_le_bytes(
                media[inner_offset + C11_HEADER + 32..inner_offset + C11_HEADER + 40]
                    .try_into()
                    .unwrap(),
            );
            if ordinal != 1 {
                continue;
            }
            let payload_len = u32::from_le_bytes(
                media[outer_offset + 24..outer_offset + 28]
                    .try_into()
                    .unwrap(),
            ) as usize;
            let inner_len = C11_HEADER
                + u32::from_le_bytes(
                    media[inner_offset + 12..inner_offset + 16]
                        .try_into()
                        .unwrap(),
                ) as usize;
            let frame_len = C5_HEADER + payload_len;
            if outer_offset + frame_len <= media.len()
                && inner_len == C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX + CHUNK_BYTES
                && payload_len
                    > C5_EXTENT_METADATA + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX
            {
                candidates.push((path.clone(), outer_offset, frame_len));
            }
        }
    }
    assert_eq!(candidates.len(), 1, "one selected ordinal-one chunk");
    let (path, outer_offset, frame_len) = candidates.pop().unwrap();
    let mut media = std::fs::read(&path).unwrap();
    let outer = &mut media[outer_offset..outer_offset + frame_len];
    let payload_byte =
        C5_HEADER + C5_EXTENT_METADATA + C11_HEADER + C11_CHUNK_CLAIM + C11_CONTENT_PREFIX;
    outer[payload_byte] ^= 0x01;
    let checksum = crc32c(&outer[..44], &outer[C5_HEADER..]);
    outer[44..C5_HEADER].copy_from_slice(&checksum.to_le_bytes());
    std::fs::write(path, media).unwrap();
}

pub(crate) fn crc32c(prefix: &[u8], payload: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in prefix.iter().chain(payload) {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !crc
}