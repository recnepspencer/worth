use std::path::Path;

use worth_store_recovery_runtime::{
    PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimits, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryPlatformAuthority, PhysicalRecoveryStaticConfiguration,
};

pub(super) fn open_request(
    root: &Path,
    profile: super::arguments::BoundedProfile,
) -> Result<PhysicalRecoveryOpenRequest, String> {
    let configuration = PhysicalRecoveryStaticConfiguration::current();
    let limits = match profile {
        super::arguments::BoundedProfile::PhaseTwoAdmission => {
            phase_two_admission_limits(512 * 1024)?
        }
        super::arguments::BoundedProfile::FateCoverage
        | super::arguments::BoundedProfile::Refused
        | super::arguments::BoundedProfile::PublicationIndeterminate => {
            phase_two_admission_limits(4 * 1024 * 1024)?
        }
    };
    let authority =
        PhysicalRecoveryPlatformAuthority::acquire(root.to_owned(), configuration.clone(), limits)
            .map_err(|error| format!("platform authority refused: {error:?}"))?;
    let backend_profile = authority.qualified_backend_profile().clone();
    Ok(PhysicalRecoveryOpenRequest::declare(
        root.to_owned(),
        configuration,
        backend_profile,
        limits,
        authority,
    ))
}

fn phase_two_admission_limits(
    recovery_memory_bytes: u64,
) -> Result<PhysicalRecoveryLimits, String> {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
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
        recovery_memory_bytes,
        dirty_frames: 1_000_000,
        concurrent_commands: 64,
        publication_effects: 256,
        cleanup_candidates: 1_000_000,
        cleanup_bytes: u32::MAX as u64,
        observation_bytes: 64 * 1024 * 1024,
    })
    .map_err(|error| format!("invalid built-in bounded profile: {error:?}"))
}
