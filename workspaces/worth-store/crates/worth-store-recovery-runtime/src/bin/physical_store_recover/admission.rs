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
        super::arguments::BoundedProfile::C11BlobCrash => c11_blob_crash_limits()?,
        super::arguments::BoundedProfile::C11BlobMultilevel => c11_blob_multilevel_limits()?,
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

/// The C11 killed-child world uses the C8 arena-crash recovery envelope for
/// source candidates, WAL/redo, staging, and cleanup. Three blob chunks plus
/// a tree/publication redo require more than the CLI's 512 KiB admission
/// profile (observed planning charge: 3,693,672 bytes); 8 MiB remains finite.
fn c11_blob_crash_limits() -> Result<PhysicalRecoveryLimits, String> {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        // Root selection and the completed baseline checkpoint precede the
        // killed blob publication; these are the C11 arena-crash source caps.
        selector_candidates: 4,
        checkpoint_candidates: 8,
        // Routed record manifests include the seed, declaration, chunks,
        // tree leaf, and recovered generation, not an unbounded catalog.
        manifest_bytes: 16 << 20,
        manifest_entries: 4096,
        // Seven WAL frames are expected in the three-chunk world; retain the
        // established arena-crash rotation and valid-prefix headroom.
        wal_segments: 8,
        wal_frames: 4096,
        wal_bytes: 16 << 20,
        // C8 may materialize routed records and a packed arena during redo.
        redo_targets: 4096,
        redo_bytes: 64 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 64 << 20,
        // Actual three-chunk planning charged 3,693,672 bytes. This finite
        // 8 MiB envelope covers that measured world without using a global
        // unbounded or phase-eight fate profile.
        recovery_memory_bytes: 8 << 20,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        // Recovery cleanup and post-recovery observation remain capped even
        // if the killed process left predecessor/staging artifacts.
        cleanup_candidates: 4096,
        cleanup_bytes: 64 << 20,
        observation_bytes: 64 << 20,
    })
    .map_err(|error| format!("invalid C11 blob-crash bounded profile: {error:?}"))
}

/// The 4,097-chunk crash world has 4,166 selected records including the seed,
/// declaration, 64 periodic frontiers and three tree nodes. Discovery charges
/// both current and previous root manifests against one limit. Every 64 chunks
/// the writer completes a physical checkpoint, keeping the post-checkpoint WAL
/// tail below the 64 MiB retained-tail policy. This profile is not the 4 GiB
/// release lane; its limits admit only this named multilevel process journey.
fn c11_blob_multilevel_limits() -> Result<PhysicalRecoveryLimits, String> {
    PhysicalRecoveryLimits::admit(PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 128,
        manifest_bytes: 32 << 20,
        manifest_entries: 16_384,
        wal_segments: 16,
        wal_frames: 4096,
        wal_bytes: 64 << 20,
        redo_targets: 8192,
        redo_bytes: 128 << 20,
        distinct_pages_and_extents: 8192,
        operation_bindings: 8192,
        staging_bytes: 128 << 20,
        recovery_memory_bytes: 32 << 20,
        dirty_frames: 8192,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 8192,
        cleanup_bytes: 128 << 20,
        observation_bytes: 128 << 20,
    })
    .map_err(|error| format!("invalid C11 blob-multilevel bounded profile: {error:?}"))
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

#[cfg(test)]
mod tests {
    use super::c11_blob_multilevel_limits;

    #[test]
    fn multilevel_world_has_an_admissible_finite_recovery_envelope() {
        c11_blob_multilevel_limits().expect("every multilevel limit is finite and nonzero");
    }
}
