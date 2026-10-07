use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, ExtentArenaId, ExtentArenaRange,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::*;
use crate::physical_runtime::recovery_construction::selected_rejoin::resident::StoreRejoinResidentLedger;
use crate::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRecoveryAllocationAdmission,
    PhysicalRuntimeAdmission, PhysicalStore, QualifiedRecoveryFilesystemMedia,
};

#[test]
fn understated_root_count_denies_before_leaf_entries_grow_maps() {
    let (directory, root, free, format) = routing_fixture(1);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path().join("store"))
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let mut discovery = media.bounded_discovery(64, 1 << 20).unwrap();
    assert!(matches!(
        verify(&mut discovery, &root, &free, format, None),
        Err(Denial::BoundExceeded)
    ));
}

#[test]
fn resident_route_walk_denies_before_read_and_charges_colive_reread() {
    let (directory, root, free, format) = routing_fixture(2);
    let media = QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path().join("store"))
        .unwrap()
        .admit_persisted_store()
        .unwrap();
    let identity = media.store_identity();
    let mut discovery = media.bounded_discovery(64, 1 << 20).unwrap();
    let mut low = StoreRejoinResidentLedger::for_test(
        PhysicalRecoveryAllocationAdmission::new(identity, 1 << 20),
        0,
        u64::from(format.page_size().bytes()) - 1,
    )
    .unwrap();
    assert!(matches!(
        verify_with_resident(&mut discovery, &root, &free, format, &mut low),
        Err(Denial::Resident(_))
    ));
    assert_eq!(discovery.counters().addressed_artifacts_read, 0);
    let media = discovery.finish();
    let mut discovery = media.bounded_discovery(64, 1 << 20).unwrap();
    let mut adequate = StoreRejoinResidentLedger::for_test(
        PhysicalRecoveryAllocationAdmission::new(identity, 1 << 20),
        0,
        1 << 20,
    )
    .unwrap();
    let first = verify_with_resident(&mut discovery, &root, &free, format, &mut adequate).unwrap();
    let first_peak = adequate.peak();
    let first_retained = adequate.used();
    let second = verify_with_resident(&mut discovery, &root, &free, format, &mut adequate).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.selected_routes().len(), 2);
    assert_eq!(discovery.counters().addressed_artifacts_read, 2);
    let media = discovery.finish();
    let mut discovery = media.bounded_discovery(64, 1 << 20).unwrap();
    assert!(first_retained > 0);
    let combined_ceiling = first_peak;
    let mut combined = StoreRejoinResidentLedger::for_test(
        PhysicalRecoveryAllocationAdmission::new(identity, 1 << 20),
        0,
        combined_ceiling,
    )
    .unwrap();
    let held = verify_with_resident(&mut discovery, &root, &free, format, &mut combined).unwrap();
    assert_eq!(held, first);
    let reads_before_second = discovery.counters().addressed_artifacts_read;
    assert!(matches!(
        verify_with_resident(&mut discovery, &root, &free, format, &mut combined),
        Err(Denial::Resident(_))
    ));
    assert_eq!(
        discovery.counters().addressed_artifacts_read,
        reads_before_second
    );
}

fn routing_fixture(
    record_count: u64,
) -> (
    tempfile::TempDir,
    DurablePhysicalRootManifest,
    DurableFreeSpaceManifestHeader,
    PhysicalRecordFormatDeclaration,
) {
    let directory = tempfile::tempdir().unwrap();
    let root_path = directory.path().join("store");
    initialize(&root_path);
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let routes = (1..=2)
        .map(|ordinal| {
            let record = PersistedRecordIdentity::new([3; 16], ordinal).unwrap();
            let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
                .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
                .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
            CurrentPhysicalRecordPlacement::Extent(
                DurableExtentRecordPlacement::legacy_unknown(
                    record,
                    extent,
                    3,
                    ExtentArenaRange::new(
                        ExtentArenaId::new(1).unwrap(),
                        (ordinal - 1) * 20_480,
                        20_480,
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let block = PhysicalRootRoutingBlock::leaf(7, 11, 1, routes, 4).unwrap();
    let bytes = block.encode(format);
    let roots = root_path.join("families/records/roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(
        roots.join(
            RecordArtifactFile::RootRoutingBlock {
                generation: 11,
                block: 1,
            }
            .file_name(),
        ),
        &bytes,
    )
    .unwrap();
    let free =
        DurableFreeSpaceManifestHeader::new(11, 7, 4, 4, 0, 1, 1, 2, 2, 65_536, 4096, 2, None)
            .unwrap();
    let root = DurablePhysicalRootManifest::builder(
        11,
        7,
        4,
        durable_artifact_checksum(&free.encode(format)),
    )
    .record_count(record_count)
    .next_block(2)
    .routing_root(Some(block.reference(durable_artifact_checksum(&bytes))))
    .free_space_root(free.root())
    .admit()
    .unwrap();
    (directory, root, free, format)
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
