pub(crate) use {std::num::NonZeroU64};
pub(crate) use {std::path::Path};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_store::physical_runtime::FilesystemAccessPosture, worth_store::physical_runtime::FilesystemMediaAdmission, worth_store::physical_runtime::PhysicalRuntimeAdmission, worth_store::physical_runtime::PhysicalStore};
pub(crate) use {worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::CheckpointBindingCompactionHeader, worth_store_physical_format::CheckpointRootBasis, worth_store_physical_format::CheckpointStreamEncoder, worth_store_physical_format::CheckpointWalSourceRange, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::FreeSpaceBlockReference, worth_store_physical_format::FreeSpaceKey, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalCheckpointIdentity, worth_store_physical_format::PhysicalCheckpointSource, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalFreeSpaceMembershipBlock, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::RecordFreeSpaceManifestEntry, worth_store_physical_format::RootSelectorIdentity, worth_store_physical_format::RootSelectorRole};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration, worth_store_recovery_runtime::PhysicalRecoveryLimits, worth_store_recovery_runtime::PhysicalRecoveryOpenRequest, worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority, worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration};



pub(crate) fn expect_blocked(
    outcome: worth_store_recovery_runtime::PhysicalRecoveryOutcome,
) -> worth_store_recovery_runtime::PhysicalRecoveryBlock {
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("persisted-source failure must be a top-level blocked outcome")
    };
    *blocked
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
    let configuration = worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::for_record_format(format);
    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
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
    let configuration = worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current();
    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.to_path_buf(),
        configuration.clone(),
        limits,
    )
    .unwrap();
    let profile = authority.qualified_backend_profile().clone();
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
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
        worth_store::physical_runtime::PhysicalStore::admit(worth_store::physical_runtime::PhysicalRuntimeAdmission::new(root.to_path_buf()).unwrap()).unwrap();
    let admission =
        worth_store::physical_runtime::FilesystemMediaAdmission::production(worth_store::physical_runtime::FilesystemAccessPosture::CoordinatedServiceAccount);
    let media = match runtime.try_admit_filesystem_media(admission).into_raw() {
        worth_proof::TransitionOutcome::Success(media) => media,
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
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    publish_synthetic_genesis_for_format(root, store, format);
}

pub(crate) fn publish_synthetic_genesis_for_format(
    root: &Path,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) {
    let free_entry = worth_store_physical_format::RecordFreeSpaceManifestEntry::arena_range(
        worth_store_physical_format::ExtentArenaRange::new(worth_store_physical_format::ExtentArenaId::new(1).unwrap(), 0, 64 << 20).unwrap(),
        1,
    )
    .unwrap();
    let free_block = worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::leaf(7, 1, 1, vec![free_entry], 4).unwrap();
    let free_block_bytes = free_block.encode(format);
    let free_space = worth_store_physical_format::DurableFreeSpaceManifestHeader::new(
        1,
        7,
        4,
        4,
        1,
        1,
        1,
        2,
        2,
        64 << 20,
        4096,
        2,
        Some(free_block.reference(durable_artifact_checksum(&free_block_bytes))),
    )
    .unwrap();
    let free_space_bytes = free_space.encode(format);
    let manifest =
        worth_store_physical_format::DurablePhysicalRootManifest::builder(1, 7, 4, durable_artifact_checksum(&free_space_bytes))
            .free_space_root(free_space.root())
            .admit()
            .unwrap();
    let selector = worth_store_physical_format::DurableRootSelector::new(
        store,
        format,
        worth_store_physical_format::RootSelectorIdentity::new(1).unwrap(),
        worth_store_physical_format::RootSelectorRole::Current,
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
            .join(worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest { generation: 1 }.file_name()),
        free_space_bytes,
    )
    .unwrap();
    std::fs::write(
        free_space_directory.join(
            worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock {
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
    let checkpoint = worth_store_physical_format::PhysicalCheckpointIdentity::new(store, std::num::NonZeroU64::new(1).unwrap());
    let source = worth_store_physical_format::PhysicalCheckpointSource::concurrent(
        checkpoint,
        worth_store_physical_format::CheckpointWalSourceRange::new(1, 2).unwrap(),
        worth_store_physical_format::CheckpointRootBasis::new(1, 7),
        1,
    );
    let (encoder, header) = worth_store_physical_format::CheckpointStreamEncoder::begin(source);
    let cutover = worth_store_physical_format::CheckpointBindingCompactionHeader::new(1, 2).unwrap();
    let (compaction, cutover_record) = encoder.begin_binding_compaction(cutover);
    let (_, footer) = compaction.finish();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&cutover_record);
    bytes.extend_from_slice(&footer);
    std::fs::write(root.join("families").join("checkpoint.current"), bytes).unwrap();
    checkpoint
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

/// The WAL suffix a synthetic checkpoint's binding cutoff covers. A cutoff
/// with no retained covered suffix and no straddling replay segment has no
/// original WAL origin, so recovery denies it before effects. The covered
/// frame keeps the two-field member shape; its binding is already in the
/// checkpoint, so the freshness sample classifies and then skips it.
pub(crate) fn publish_synthetic_covered_wal(root: &Path) {
    let families = root.join("families");
    let mut payload = Vec::new();
    for field in [
        &b"phase-three-covered-binding"[..],
        b"phase-three-covered-redo",
    ] {
        payload.extend_from_slice(&(field.len() as u64).to_le_bytes());
        payload.extend_from_slice(field);
    }
    let (path, bytes) =
        worth_store_test_support::harness::recovery::wal_tail::prepare_persisted_wal_frame(
            &families,
            1,
            1,
            2,
            "phase-three-covered-frame",
            &payload,
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
    worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(limit_declaration(
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