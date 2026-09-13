use std::num::NonZeroU64;
use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRuntimeAdmission, PhysicalStore,
};
use worth_store_physical_format::{
    durable_artifact_checksum, CheckpointBindingCompactionHeader, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, CurrentPhysicalRecordPlacement,
    DurableExtentRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, FreeSpaceBlockReference, FreeSpaceKey, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource, PhysicalExtentId,
    PhysicalFreeSpaceMembershipBlock, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock, RecordAllocationClass,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, RootSelectorIdentity, RootSelectorRole,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};

pub(crate) fn expect_blocked(
    outcome: worth_store_recovery_runtime::PhysicalRecoveryOutcome,
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("persisted-source failure must be a top-level blocked outcome")
    };
    blocked
}

pub(crate) fn admitted_recovery(
    root: &Path,
) -> worth_store_recovery_runtime::AdmittedPhysicalRecovery {
    admitted_recovery_with_limits(root, limits())
}

pub(crate) fn admitted_recovery_for_format(
    root: &Path,
    format: PhysicalRecordFormatDeclaration,
) -> worth_store_recovery_runtime::AdmittedPhysicalRecovery {
    let limits = limits();
    let configuration = PhysicalRecoveryStaticConfiguration::for_record_format(format);
    let authority = PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
    .admit()
    .unwrap()
}

pub(crate) fn admitted_recovery_with_limits(
    root: &Path,
    limits: PhysicalRecoveryLimits,
) -> worth_store_recovery_runtime::AdmittedPhysicalRecovery {
    recovery_request_with_limits(root, limits).admit().unwrap()
}

pub(crate) fn recovery_request_with_limits(
    root: &Path,
    limits: PhysicalRecoveryLimits,
) -> PhysicalRecoveryOpenRequest {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let authority = PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(
        root.to_path_buf(),
        configuration,
        profile,
        limits,
        authority,
    )
}

pub(crate) fn initialize_store(
    root: &Path,
) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.to_path_buf()).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("ordinary media initialization failed"),
    };
    let identity = media.store_identity();
    let _ = media.close();
    identity
}

pub(crate) fn publish_synthetic_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    publish_synthetic_genesis_for_format(root, store, format);
}

pub(crate) fn publish_synthetic_genesis_for_format(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) {
    let free_entry =
        RecordFreeSpaceManifestEntry::new(RecordAllocationClass::Extent, 1, 1, 1, 1).unwrap();
    let free_block = PhysicalFreeSpaceMembershipBlock::leaf(7, 1, 1, vec![free_entry], 4).unwrap();
    let free_block_bytes = free_block.encode(format);
    let free_space = DurableFreeSpaceManifestHeader::new(
        1,
        7,
        4,
        4,
        1,
        1,
        1,
        2,
        2,
        Some(free_block.reference(durable_artifact_checksum(&free_block_bytes))),
    )
    .unwrap();
    let free_space_bytes = free_space.encode(format);
    let manifest =
        DurablePhysicalRootManifest::builder(1, 7, 4, durable_artifact_checksum(&free_space_bytes))
            .free_space_root(free_space.root())
            .admit()
            .unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let records = root.join("families").join("records");
    let roots = records.join("roots");
    let free_space_directory = records.join("free-space");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::create_dir_all(&free_space_directory).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        manifest.encode(format),
    )
    .unwrap();
    std::fs::write(
        free_space_directory
            .join(RecordArtifactFile::FreeSpaceManifest { generation: 1 }.file_name()),
        free_space_bytes,
    )
    .unwrap();
    std::fs::write(
        free_space_directory.join(
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation: 1,
                block: 1,
            }
            .file_name(),
        ),
        free_block_bytes,
    )
    .unwrap();
}

pub(crate) fn publish_synthetic_checkpoint(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) -> PhysicalCheckpointIdentity {
    let checkpoint = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let source = PhysicalCheckpointSource::concurrent(
        checkpoint,
        CheckpointWalSourceRange::new(1, 2).unwrap(),
        CheckpointRootBasis::new(1, 7),
        1,
    );
    let (encoder, header) = CheckpointStreamEncoder::begin(source);
    let cutover = CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
    checkpoint
}

pub(crate) fn publish_synthetic_nonempty_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let placements = [1_u64, 2]
        .into_iter()
        .map(|ordinal| {
            let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
            let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
                .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
                .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
            CurrentPhysicalRecordPlacement::Extent(
                DurableExtentRecordPlacement::new(record, extent, 23).unwrap(),
            )
        })
        .collect();
    let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, placements, 4).unwrap();
    let block_bytes = block.encode(format);
    let reference = block.reference(durable_artifact_checksum(&block_bytes));
    let free_key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 4, 0x8a9b_acbd)
        .record_count(2)
        .next_block(2)
        .routing_root(Some(reference))
        .free_space_root(Some(free_space))
        .admit()
        .unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let records = root.join("families").join("records");
    let roots = records.join("roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        manifest.encode(format),
    )
    .unwrap();
    std::fs::write(
        roots.join("root-0000000000000001-block-0000000000000001.manifest"),
        block_bytes,
    )
    .unwrap();
}

pub(crate) fn publish_synthetic_branched_genesis(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let left = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 2).unwrap();
    let right =
        PhysicalRootRoutingBlock::leaf(7, 1, 2, vec![placement(2), placement(3)], 2).unwrap();
    let left_bytes = left.encode(format);
    let right_bytes = right.encode(format);
    let left_reference = left.reference(durable_artifact_checksum(&left_bytes));
    let right_reference = right.reference(durable_artifact_checksum(&right_bytes));
    let branch =
        PhysicalRootRoutingBlock::branch(7, 1, 3, 1, vec![left_reference, right_reference], 2)
            .unwrap();
    let branch_bytes = branch.encode(format);
    let branch_reference = branch.reference(durable_artifact_checksum(&branch_bytes));
    let free_key = FreeSpaceKey::new(RecordAllocationClass::Extent, 1).unwrap();
    let free_space =
        FreeSpaceBlockReference::new(1, 1, 0, 0x0102_0304, free_key, free_key).unwrap();
    let manifest = DurablePhysicalRootManifest::builder(1, 7, 2, 0x8a9b_acbd)
        .record_count(3)
        .next_block(4)
        .routing_root(Some(branch_reference))
        .free_space_root(Some(free_space))
        .admit()
        .unwrap();
    let selector = DurableRootSelector::new(
        store,
        format,
        RootSelectorIdentity::new(1).unwrap(),
        RootSelectorRole::Current,
        1,
        None,
        None,
    )
    .unwrap();
    let records = root.join("families").join("records");
    let roots = records.join("roots");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::write(records.join("root-current.selector"), selector.encode()).unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        manifest.encode(format),
    )
    .unwrap();
    for (block, bytes) in [(1, left_bytes), (2, right_bytes), (3, branch_bytes)] {
        std::fs::write(
            roots.join(format!("root-0000000000000001-block-{block:016}.manifest")),
            bytes,
        )
        .unwrap();
    }
}

fn placement(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new(record, extent, 23).unwrap(),
    )
}

pub(crate) fn publish_synthetic_wal_tail(root: &Path) {
    let families = root.join("families");
    let (path, bytes) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            2,
            3,
            "phase-three-frame",
            b"phase-three-payload",
        );
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

pub(crate) fn limits() -> PhysicalRecoveryLimits {
    limits_for(2, 8, 8 * 1024)
}

pub(crate) fn limits_for(
    selector_candidates: u64,
    wal_segments: u64,
    manifest_bytes: u64,
) -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(limit_declaration(
        selector_candidates,
        wal_segments,
        manifest_bytes,
    ))
    .unwrap()
}

pub(crate) fn limit_declaration(
    selector_candidates: u64,
    wal_segments: u64,
    manifest_bytes: u64,
) -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates,
        checkpoint_candidates: 8,
        manifest_bytes,
        manifest_entries: 8,
        wal_segments,
        wal_frames: 64,
        wal_bytes: 32 * 1024,
        redo_targets: 8,
        redo_bytes: 32 * 1024,
        distinct_pages_and_extents: 8,
        operation_bindings: 8,
        staging_bytes: 32 * 1024,
        recovery_memory_bytes: 1024 * 1024,
        dirty_frames: 8,
        concurrent_commands: 8,
        publication_effects: 4,
        cleanup_candidates: 8,
        cleanup_bytes: 32 * 1024,
        observation_bytes: 8 * 1024,
    }
}
