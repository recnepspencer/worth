use std::path::Path;

use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};

pub(super) fn open_request(root: &Path) -> Result<PhysicalRecoveryOpenRequest, String> {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 64 * 1024 * 1024,
        manifest_entries: 1_000_000,
        wal_segments: 4_096,
        wal_frames: 16_000_000,
        wal_bytes: u32::MAX as u64,
        redo_targets: 16_000_000,
        redo_bytes: u32::MAX as u64,
        distinct_pages_and_extents: 16_000_000,
        operation_bindings: 16_000_000,
        staging_bytes: u32::MAX as u64,
        recovery_memory_bytes: 64 * 1024 * 1024,
        dirty_frames: 1_000_000,
        concurrent_commands: 64,
        publication_effects: 256,
        cleanup_candidates: 1_000_000,
        cleanup_bytes: u32::MAX as u64,
        observation_bytes: 64 * 1024 * 1024,
    })
    .map_err(|error| format!("admit recovery limits: {error:?}"))?;
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .map_err(|error| format!("acquire recovery authority: {error:?}"))?;
    let backend = authority.qualified_backend_profile().clone();
    Ok(PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend,
        limits,
        authority,
    ))
}
