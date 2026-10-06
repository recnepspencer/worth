//! A successor candidate whose reader ran out of observation bytes met a
//! limit. One whose artifact outgrew the ceiling of its own read is damaged,
//! and names no limit an operator could raise.

use worth_store::physical_runtime::FilesystemObservationBound;
use worth_store_physical_format::RecordArtifactFile;

use super::*;
use crate::orchestration::reader_limit::refused_past;
use crate::orchestration::recovery_budget::recovery_limit_for_test;

fn observation_bytes(observed: u64, admitted: u64) -> Option<PhysicalRecoveryLimitFailure> {
    Some(
        recovery_limit_for_test(
            PhysicalRecoveryLimitDimension::ObservationBytes,
            observed,
            admitted,
        )
        .into(),
    )
}

fn budget() -> ManifestEntryBudget {
    ManifestEntryBudget::new(500, 0)
}

/// A candidate window that refused nothing.
fn window() -> PlanningResidentAllowance {
    PlanningResidentAllowance::new(0, 1_000).expect("an empty window")
}

fn limits() -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 4,
        checkpoint_candidates: 64,
        manifest_bytes: 1 << 20,
        manifest_entries: 500,
        wal_segments: 64,
        wal_frames: 4096,
        wal_bytes: 1 << 20,
        redo_targets: 4096,
        redo_bytes: 1 << 20,
        distinct_pages_and_extents: 4096,
        operation_bindings: 4096,
        staging_bytes: 1 << 20,
        recovery_memory_bytes: 1 << 20,
        dirty_frames: 4096,
        concurrent_commands: 8,
        publication_effects: 64,
        cleanup_candidates: 4096,
        cleanup_bytes: 1 << 20,
        observation_bytes: 10_000,
    }
}

fn outgrown(bound: FilesystemObservationBound) -> PhysicalRecoverySuccessorCandidateDenial {
    PhysicalRecoverySuccessorCandidateDenial::Discovery {
        artifact: RecordArtifactFile::RootManifest { generation: 9 },
        generation: 9,
        failure: refused_past(bound, 4_001, 4_000),
    }
}

#[test]
fn only_a_candidate_reader_out_of_its_own_budget_names_a_limit() {
    let limits = limits();
    assert_eq!(
        candidate_limit(
            &limits,
            &outgrown(FilesystemObservationBound::ObservationBytes),
            &budget(),
            &window()
        ),
        // Handed 4,000 of 10,000 bytes, the reader observed 4,001.
        observation_bytes(10_001, 10_000),
    );
    assert_eq!(
        candidate_limit(
            &limits,
            &outgrown(FilesystemObservationBound::RequestedBytes),
            &budget(),
            &window()
        ),
        None,
    );
}

#[test]
fn a_candidate_out_of_memory_or_entries_names_that_limit_with_its_counts() {
    let artifact = RecordArtifactFile::RootManifest { generation: 9 };
    // Handed 1,000 of recovery's memory, the candidate needed 1,001. The
    // denial names no count: the window holds both.
    let memory = PhysicalRecoverySuccessorCandidateDenial::RecoveryMemoryBytes {
        artifact,
        generation: 9,
    };
    let mut refused = window();
    assert!(refused.transient(1_001).is_err());
    let held = (1 << 20) - 1_000;
    assert_eq!(
        candidate_limit(&limits(), &memory, &budget(), &refused),
        Some(
            recovery_limit_for_test(
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
                held + 1_001,
                1 << 20,
            )
            .into()
        ),
    );
    // A window that refused nothing, as when a count overflowed, states none.
    assert_eq!(
        candidate_limit(&limits(), &memory, &budget(), &window()),
        None
    );
    let entries = PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit {
        artifact,
        generation: 9,
    };
    let mut spent = budget();
    assert!(spent.charge(501).is_err());
    assert_eq!(
        candidate_limit(&limits(), &entries, &spent, &window()),
        Some(
            recovery_limit_for_test(PhysicalRecoveryLimitDimension::ManifestEntries, 501, 500)
                .into()
        )
    );
}

/// A damaged or conflicting candidate names no limit, even beside budgets
/// that refused.
#[test]
fn a_damaged_candidate_names_no_limit() {
    use crate::entry::{
        PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySuccessorCandidateMismatch,
    };
    use PhysicalRecoverySuccessorCandidateDenial as Candidate;
    let artifact = RecordArtifactFile::RootManifest { generation: 9 };
    let cause = Vec::<u8>::new().try_reserve(usize::MAX).unwrap_err();
    let mut spent = budget();
    assert!(spent.charge(501).is_err());
    let mut refused = window();
    assert!(refused.transient(1_001).is_err());
    for damage in [
        Candidate::Allocation {
            artifact,
            generation: 9,
            requested_bytes: 5,
            cause,
        },
        Candidate::MissingArtifact {
            artifact,
            generation: 9,
        },
        Candidate::InvalidArtifact {
            artifact,
            generation: 9,
        },
        Candidate::RootProtocol {
            artifact,
            generation: 9,
            denial: PhysicalRecoveryRootProtocolDenial::Absent,
        },
        Candidate::Conflict {
            artifact,
            generation: 9,
            mismatch: PhysicalRecoverySuccessorCandidateMismatch::SuccessorArtifactBytes,
        },
    ] {
        assert_eq!(
            candidate_limit(&limits(), &damage, &spent, &refused),
            None,
            "{damage:?}"
        );
    }
}
