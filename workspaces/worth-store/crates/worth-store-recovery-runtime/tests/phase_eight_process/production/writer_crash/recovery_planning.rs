use std::path::Path;

use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryStaticConfiguration, PlannedPhysicalRecovery,
};

pub(super) fn plan_with_memory(
    root: &Path,
    recovery_memory_bytes: u64,
) -> Result<PlannedPhysicalRecovery, PhysicalRecoveryOutcome> {
    plan_with_limits(root, successor_limits(recovery_memory_bytes))
}

pub(super) fn plan_with_limits(
    root: &Path,
    limits: PhysicalRecoveryLimits,
) -> Result<PlannedPhysicalRecovery, PhysicalRecoveryOutcome> {
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
    .admit()
    .unwrap()
    .discover()
    .unwrap()
    .select()
    .unwrap()
    .plan()
}

pub(super) fn successor_limits(recovery_memory_bytes: u64) -> PhysicalRecoveryLimits {
    successor_limits_with_observation(recovery_memory_bytes, 64 * 1024 * 1024)
}

pub(super) fn successor_limits_with_observation(
    recovery_memory_bytes: u64,
    observation_bytes: u64,
) -> PhysicalRecoveryLimits {
    successor_limits_with_manifest_and_observation(recovery_memory_bytes, 4_096, observation_bytes)
}

pub(super) fn successor_limits_with_manifest_entries(
    recovery_memory_bytes: u64,
    manifest_entries: u64,
) -> PhysicalRecoveryLimits {
    successor_limits_with_manifest_and_observation(
        recovery_memory_bytes,
        manifest_entries,
        64 * 1024 * 1024,
    )
}

fn successor_limits_with_manifest_and_observation(
    recovery_memory_bytes: u64,
    manifest_entries: u64,
    observation_bytes: u64,
) -> PhysicalRecoveryLimits {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 8,
        checkpoint_candidates: 8,
        manifest_bytes: 64 * 1024 * 1024,
        manifest_entries,
        wal_segments: 64,
        wal_frames: 4_096,
        wal_bytes: 16 * 1024 * 1024,
        redo_targets: 4_096,
        redo_bytes: 16 * 1024 * 1024,
        distinct_pages_and_extents: 4_096,
        operation_bindings: 4_096,
        staging_bytes: 64 * 1024 * 1024,
        recovery_memory_bytes,
        dirty_frames: 4_096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 8,
        cleanup_bytes: 32 * 1024,
        observation_bytes,
    })
    .unwrap()
}
