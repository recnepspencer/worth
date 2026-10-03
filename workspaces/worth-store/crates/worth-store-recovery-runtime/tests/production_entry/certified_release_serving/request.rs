use super::*;
use worth_store::physical_runtime::AdmittedPhysicalRecordFormat;

pub(crate) fn request(root: &Path) -> PhysicalRecoveryOpenRequest {
    request_with_memory(root, 16 << 20)
}

pub(crate) fn request_with_memory(
    root: &Path,
    recovery_memory_bytes: u64,
) -> PhysicalRecoveryOpenRequest {
    request_with_configuration(
        root,
        recovery_memory_bytes,
        PhysicalRecoveryStaticConfiguration::current(),
    )
}

pub(crate) fn request_with_memory_and_format(
    root: &Path,
    recovery_memory_bytes: u64,
    format: AdmittedPhysicalRecordFormat,
) -> PhysicalRecoveryOpenRequest {
    request_with_configuration(
        root,
        recovery_memory_bytes,
        PhysicalRecoveryStaticConfiguration::for_record_format(format.declaration()),
    )
}

pub(crate) fn request_with_configuration(
    root: &Path,
    recovery_memory_bytes: u64,
    configuration: PhysicalRecoveryStaticConfiguration,
) -> PhysicalRecoveryOpenRequest {
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 64 << 20,
        manifest_entries: 4096,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 64 << 20,
        redo_targets: 4096,
        redo_bytes: 64 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 64 << 20,
        recovery_memory_bytes,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 64 << 20,
        observation_bytes: 64 << 20,
    })
    .expect("bounded certification limits");
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .expect("fresh recovery platform authority");
    let backend = authority.qualified_backend_profile().clone();
    PhysicalRecoveryOpenRequest::declare(root.to_owned(), configuration, backend, limits, authority)
}
