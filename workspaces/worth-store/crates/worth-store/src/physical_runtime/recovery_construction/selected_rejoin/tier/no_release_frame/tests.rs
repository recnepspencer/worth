use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    encode_extent_chunk, DurableExtentManifest, DurableExtentRecordPlacement,
    ExtentArenaFrameLayout, ExtentArenaId, ExtentArenaRange, ExtentChunkCoordinate,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, RecordArtifactFile, DURABLE_EXTENT_FRAME_HEADER_BYTES,
    EXTENT_CHUNK_METADATA_BYTES,
};

use super::*;
use crate::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};

const CHUNKS: u32 = 16;

#[test]
fn valid_many_chunk_extent_denies_before_excess_slice_allocation() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("store");
    initialize(&root);
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let (placement, payload) = write_extent(&root, format);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(&root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(128, 2 << 20).unwrap();
    let mut slices = Vec::new();
    assert!(matches!(
        read_with_slice_limit(
            &mut discovery,
            format,
            placement,
            payload.len() as u64,
            &mut slices,
            CHUNKS as usize,
        ),
        Err(Denial::BoundExceeded)
    ));
    assert!(
        slices.is_empty(),
        "the validated manifest's chunk count must deny before any slice append"
    );
    assert_eq!(
        read_with_slice_limit(
            &mut discovery,
            format,
            placement,
            payload.len() as u64,
            &mut slices,
            CHUNKS as usize + 1,
        )
        .unwrap(),
        payload
    );
    assert_eq!(slices.len(), CHUNKS as usize + 1);
}

fn initialize(root: &Path) {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_owned()).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fixture store initialization failed"),
    };
    let _ = media.close();
}

fn write_extent(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
) -> (DurableExtentRecordPlacement, Vec<u8>) {
    let record = PersistedRecordIdentity::new([7; 16], 1).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let alignment = u64::from(format.page_size().bytes());
    let layout = ExtentArenaFrameLayout::new(format, alignment).unwrap();
    let chunk_capacity = format.page_size().bytes() as usize
        - DURABLE_EXTENT_FRAME_HEADER_BYTES
        - EXTENT_CHUNK_METADATA_BYTES;
    let payload = vec![0x4a; chunk_capacity * (CHUNKS as usize - 1) + 5];
    let manifest = DurableExtentManifest::new(
        format,
        record,
        extent,
        payload.len() as u64,
        format.page_size().bytes(),
        CHUNKS,
        alignment,
    )
    .unwrap();
    let range = ExtentArenaRange::new(
        ExtentArenaId::new(1).unwrap(),
        0,
        layout.allocated_bytes(CHUNKS).unwrap(),
    )
    .unwrap();
    let placement =
        DurableExtentRecordPlacement::legacy_unknown(record, extent, payload.len() as u64, range)
            .unwrap();
    let mut arena = vec![0; usize::try_from(range.length()).unwrap()];
    let manifest_bytes = manifest.encode(format);
    arena[..manifest_bytes.len()].copy_from_slice(&manifest_bytes);
    for ordinal in 1..=CHUNKS {
        let offset = (ordinal as usize - 1) * chunk_capacity;
        let end = payload.len().min(offset + chunk_capacity);
        let coordinate = ExtentChunkCoordinate::new(
            record,
            extent,
            payload.len() as u64,
            offset as u64,
            ordinal,
        )
        .unwrap();
        let bytes = encode_extent_chunk(format, coordinate, &payload[offset..end]).unwrap();
        let frame_offset = layout.chunk_offset(ordinal).unwrap() as usize;
        arena[frame_offset..frame_offset + bytes.len()].copy_from_slice(&bytes);
    }
    let arena_path = root
        .join("families/records/arenas")
        .join(RecordArtifactFile::ExtentArena { arena: 1 }.file_name());
    std::fs::create_dir_all(arena_path.parent().unwrap()).unwrap();
    std::fs::write(arena_path, arena).unwrap();
    (placement, payload)
}
