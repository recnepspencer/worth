use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, ExtentArenaId, ExtentArenaRange,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalFreeSpaceMembershipBlock,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalInventoryTranscriptV1,
    PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock, RecordArtifactFile,
    RecordFreeSpaceManifestEntry,
};

use super::*;
use crate::physical_runtime::recovery_construction::selected_rejoin::tier;
use crate::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};

#[test]
fn independently_valid_same_count_route_and_free_substitution_denied_before_seal() {
    let directory = tempfile::tempdir().unwrap();
    let store_root = directory.path().join("store");
    initialize(&store_root);
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let original = write_snapshot(&store_root, format, 11, 0, 28672);
    let substituted = write_snapshot(&store_root, format, 12, 4096, 32768);
    let original_transcript = rewalk(&store_root, format, &original.0, &original.1);
    let substituted_transcript = rewalk(&store_root, format, &substituted.0, &substituted.1);

    assert_eq!(
        original_transcript.route_count(),
        substituted_transcript.route_count()
    );
    assert_eq!(
        original_transcript.free_entry_count(),
        substituted_transcript.free_entry_count()
    );
    assert_ne!(
        original_transcript.routes_sha256(),
        substituted_transcript.routes_sha256()
    );
    assert_ne!(
        original_transcript.free_entries_sha256(),
        substituted_transcript.free_entries_sha256()
    );
    assert!(matches!(
        require_transcript(original_transcript, substituted_transcript),
        Err(Denial::RoutingFrame)
    ));
    assert!(require_transcript(substituted_transcript, substituted_transcript).is_ok());
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

fn write_snapshot(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    generation: u64,
    route_offset: u64,
    free_offset: u64,
) -> (DurablePhysicalRootManifest, DurableFreeSpaceManifestHeader) {
    let record = PersistedRecordIdentity::new([3; 16], 1).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(1).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let route = CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(
            record,
            extent,
            3,
            ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), route_offset, 20480).unwrap(),
        )
        .unwrap(),
    );
    let routing = PhysicalRootRoutingBlock::leaf(7, generation, 1, vec![route], 4).unwrap();
    let routing_bytes = routing.encode(format);
    let free_entry = RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), free_offset, 4096).unwrap(),
        generation,
    )
    .unwrap();
    let membership =
        PhysicalFreeSpaceMembershipBlock::leaf(7, generation, 1, vec![free_entry], 4).unwrap();
    let membership_bytes = membership.encode(format);
    let free = DurableFreeSpaceManifestHeader::new(
        generation,
        7,
        4,
        4,
        1,
        1,
        1,
        2,
        2,
        65536,
        4096,
        2,
        Some(membership.reference(durable_artifact_checksum(&membership_bytes))),
    )
    .unwrap();
    let root = DurablePhysicalRootManifest::builder(
        generation,
        7,
        4,
        durable_artifact_checksum(&free.encode(format)),
    )
    .record_count(1)
    .next_block(2)
    .routing_root(Some(
        routing.reference(durable_artifact_checksum(&routing_bytes)),
    ))
    .free_space_root(free.root())
    .admit()
    .unwrap();
    let roots = store_root.join("families").join("records").join("roots");
    let free_space = store_root
        .join("families")
        .join("records")
        .join("free-space");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::create_dir_all(&free_space).unwrap();
    std::fs::write(
        roots.join(
            RecordArtifactFile::RootRoutingBlock {
                generation,
                block: 1,
            }
            .file_name(),
        ),
        routing_bytes,
    )
    .unwrap();
    std::fs::write(
        free_space.join(
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation,
                block: 1,
            }
            .file_name(),
        ),
        membership_bytes,
    )
    .unwrap();
    (root, free)
}

fn rewalk(
    store_root: &Path,
    format: PhysicalRecordFormatDeclaration,
    root: &DurablePhysicalRootManifest,
    free: &DurableFreeSpaceManifestHeader,
) -> PhysicalInventoryTranscriptV1 {
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(store_root)
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(64, 1 << 20).unwrap();
    let mut transcript = PhysicalInventoryTranscriptBuilderV1::new(root, free, format, 64).unwrap();
    let routes =
        tier::routes::verify_transcribed(&mut discovery, root, free, format, &mut transcript)
            .expect("independently valid rooted routing tree");
    let memberships = observe_memberships(&mut discovery, root, free, format, &mut transcript)
        .expect("independently valid rooted free-membership tree");
    assert_eq!(routes.selected_routes().len(), 1);
    assert_eq!(memberships.len(), 1);
    transcript.finish().unwrap()
}
