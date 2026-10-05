//! A successor candidate whose reader ran out of observation bytes met a
//! limit. One whose artifact outgrew the ceiling of its own read is damaged,
//! and names no limit an operator could raise.

use worth_store::physical_runtime::FilesystemObservationBound;
use worth_store_physical_format::RecordArtifactFile;

use super::*;
use crate::orchestration::reader_limit::refused_past;

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
            4_000
        ),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: 10_001,
            admitted: 10_000,
        }),
    );
    assert_eq!(
        candidate_limit(
            &limits,
            &outgrown(FilesystemObservationBound::RequestedBytes),
            4_000
        ),
        None,
    );
}

#[test]
fn a_candidate_attempt_with_no_observation_bytes_left_names_that_limit() {
    // No reader opens on nothing: every admitted byte was already observed.
    let denial = PhysicalRecoverySuccessorCandidateDenial::ObservationBytesExhausted {
        artifact: RecordArtifactFile::RootManifest { generation: 9 },
        generation: 9,
    };
    assert_eq!(
        candidate_limit(&limits(), &denial, 0),
        Some(PhysicalRecoveryLimitFailure {
            dimension: PhysicalRecoveryLimitDimension::ObservationBytes,
            observed: 10_001,
            admitted: 10_000,
        }),
    );
}
