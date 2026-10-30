pub use worth_store::physical_runtime::AdmittedPhysicalRecordFormat;
pub use {
    crate::production_entry::certified_release_serving::serving_durability, std::num::NonZeroU64,
    std::path::Path, worth_store_recovery_runtime::PhysicalRecoveryLimitDeclaration,
    worth_store_recovery_runtime::PhysicalRecoveryLimits,
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest,
    worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority,
    worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration,
};

pub fn request(root: &Path) -> PhysicalRecoveryOpenRequest {
    request_with_memory(root, 16 << 20)
}

pub fn request_with_memory(root: &Path, recovery_memory_bytes: u64) -> PhysicalRecoveryOpenRequest {
    request_with_configuration(
        root,
        recovery_memory_bytes,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current()
            .with_durability_declaration(serving_durability(
                std::num::NonZeroU64::new(16 << 20).unwrap(),
            )),
    )
}

pub fn request_with_memory_and_format(
    root: &Path,
    recovery_memory_bytes: u64,
    format: AdmittedPhysicalRecordFormat,
) -> PhysicalRecoveryOpenRequest {
    request_with_configuration(
        root,
        recovery_memory_bytes,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::for_record_format(
            format.declaration(),
        ),
    )
}

pub fn request_with_configuration(
    root: &Path,
    recovery_memory_bytes: u64,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> PhysicalRecoveryOpenRequest {
    declare(root, recovery_memory_bytes, 4096, configuration)
}

/// Recovery of a world whose WAL holds one whole 66-chunk ingest and whose
/// release batches fill a manifest. The WAL payloads alone pass the memory of
/// the two-chunk worlds, and proving 64 dropped records absent from a result
/// root reads more manifest entries than those worlds ever route. The ingest
/// also leaves about 140 root generations above the checkpoint, and recovery
/// re-reads the manifest of each one to order the retirements among them.
/// Fresh object/session identities can change durable index packing and the
/// byte footprint between producers. Replanning fixed media is deterministic.
/// This fixture chooses 128 MiB: twice its own 64 MiB ADMITTED_BYTES window,
/// about three times the measured 43 MiB planning need. That headroom covers
/// producer-dependent packing; it is not a bound declared by recovery.
pub fn request_for_long_ingest(root: &Path) -> PhysicalRecoveryOpenRequest {
    request_for_long_ingest_with_manifest_entries(root, 18432)
}

/// The limits of the long-ingest worlds with exactly this many manifest
/// entries.
pub fn request_for_long_ingest_with_manifest_entries(
    root: &Path,
    manifest_entries: u64,
) -> PhysicalRecoveryOpenRequest {
    let mut limits = declaration(ADMITTED_BYTES, manifest_entries);
    limits.recovery_memory_bytes = limits
        .observation_bytes
        .checked_add(limits.staging_bytes)
        .expect("finite observation and planning headroom");
    open(
        root,
        limits,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
    )
}

/// The limits of the two-chunk worlds with exactly this many manifest entries.
pub fn request_with_manifest_entries(
    root: &Path,
    manifest_entries: u64,
) -> PhysicalRecoveryOpenRequest {
    request_narrowing(root, |declared| {
        declared.manifest_entries = manifest_entries
    })
}

/// The bytes every request of these worlds admits of each byte limit, but
/// for recovery memory.
pub const ADMITTED_BYTES: u64 = 64 << 20;

/// The limits of the two-chunk worlds, but for what `narrow` declares.
pub fn request_narrowing(
    root: &Path,
    narrow: impl FnOnce(&mut PhysicalRecoveryLimitDeclaration),
) -> PhysicalRecoveryOpenRequest {
    let mut declaration = declaration(16 << 20, 4096);
    narrow(&mut declaration);
    open(
        root,
        declaration,
        worth_store_recovery_runtime::PhysicalRecoveryStaticConfiguration::current(),
    )
}

pub fn declare(
    root: &Path,
    recovery_memory_bytes: u64,
    manifest_entries: u64,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> PhysicalRecoveryOpenRequest {
    open(
        root,
        declaration(recovery_memory_bytes, manifest_entries),
        configuration,
    )
}

pub fn declaration(
    recovery_memory_bytes: u64,
    manifest_entries: u64,
) -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: ADMITTED_BYTES,
        manifest_entries,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: ADMITTED_BYTES,
        redo_targets: 4096,
        redo_bytes: ADMITTED_BYTES,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: ADMITTED_BYTES,
        recovery_memory_bytes,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: ADMITTED_BYTES,
        observation_bytes: ADMITTED_BYTES,
    }
}

pub fn open(
    root: &Path,
    declaration: PhysicalRecoveryLimitDeclaration,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> PhysicalRecoveryOpenRequest {
    let limits = worth_store_recovery_runtime::PhysicalRecoveryLimits::admit(declaration)
        .expect("bounded certification limits");
    let authority = worth_store_recovery_runtime::PhysicalRecoveryPlatformAuthority::acquire(
        root.to_owned(),
        configuration.clone(),
        limits,
    )
    .expect("fresh recovery platform authority");
    let backend = authority.qualified_backend_profile().clone();
    worth_store_recovery_runtime::PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    )
}
