use worth_proof::TransitionOutcome;
use worth_signal::facade::{
    ClockAdvanceOrdinal, ClockCheckpointId, ClockDomain, ClockTick, TemporalWakeId, WakeOrdinal,
};

use crate::facade::{
    AdmittedBridgeTemporalBasis, BridgeTemporalBasisDenial, BridgeTemporalCdcCursorIdentity,
    BridgeTemporalSignalBasis, BridgeTemporalTruthViewBasis, BridgeTemporalWakeEvidence,
};

#[test]
fn temporal_basis_admission_is_stable_for_equal_inputs() {
    let truth_basis = BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let signal_basis = BridgeTemporalSignalBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        ClockDomain::MonotonicExecution,
        ClockTick::new(7),
        ClockAdvanceOrdinal::new(3),
        Some(ClockCheckpointId::new(2)),
    );
    let wake = BridgeTemporalWakeEvidence::new(
        TemporalWakeId::new(11),
        WakeOrdinal::new(5),
        ClockTick::new(7),
    );

    let left = admit(
        truth_basis.clone(),
        signal_basis.clone(),
        Some(wake.clone()),
    );
    let right = admit(truth_basis, signal_basis, Some(wake));

    assert_eq!(left, right);
    assert_eq!(left.canonical_basis_text(), right.canonical_basis_text());
    assert_eq!(
        left.truth_basis().canonical_digest(),
        right.truth_basis().canonical_digest()
    );
    assert_eq!(
        left.signal_basis().canonical_digest(),
        right.signal_basis().canonical_digest()
    );
    assert_eq!(
        left.wake_evidence().canonical_digest(),
        right.wake_evidence().canonical_digest()
    );
}

#[test]
fn temporal_basis_denies_branch_mismatch() {
    let truth_basis = BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let signal_basis = BridgeTemporalSignalBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-b"),
        ClockDomain::MonotonicExecution,
        ClockTick::new(3),
        ClockAdvanceOrdinal::new(1),
        None,
    );

    let outcome = AdmittedBridgeTemporalBasis::admit(
        truth_basis,
        signal_basis,
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(12),
            WakeOrdinal::new(2),
            ClockTick::new(3),
        )),
    );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::BranchMismatch { .. })
    ));
}

#[test]
fn temporal_basis_denies_missing_wake_evidence() {
    let truth_basis = BridgeTemporalTruthViewBasis::branch_head(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-head"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let signal_basis = BridgeTemporalSignalBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        ClockDomain::MonotonicExecution,
        ClockTick::new(4),
        ClockAdvanceOrdinal::new(2),
        None,
    );

    let outcome = AdmittedBridgeTemporalBasis::admit(truth_basis, signal_basis, None);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::MissingWakeEvidence)
    );
}

#[test]
fn temporal_basis_denies_regressed_wake_tick() {
    let outcome = AdmittedBridgeTemporalBasis::admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            ClockDomain::MonotonicExecution,
            ClockTick::new(9),
            ClockAdvanceOrdinal::new(4),
            None,
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(22),
            WakeOrdinal::new(5),
            ClockTick::new(8),
        )),
    );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::WakeTickRegressed {
            signal_clock_tick: 9,
            wake_tick: 8,
        })
    ));
}

#[test]
fn temporal_basis_keeps_historical_and_branch_head_distinct() {
    let signal_basis = BridgeTemporalSignalBasis::new(
        crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
        ClockDomain::MonotonicExecution,
        ClockTick::new(10),
        ClockAdvanceOrdinal::new(4),
        Some(ClockCheckpointId::new(9)),
    );
    let wake = BridgeTemporalWakeEvidence::new(
        TemporalWakeId::new(13),
        WakeOrdinal::new(6),
        ClockTick::new(10),
    );
    let branch_head = admit(
        BridgeTemporalTruthViewBasis::branch_head(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        signal_basis.clone(),
        Some(wake.clone()),
    );
    let historical = admit(
        BridgeTemporalTruthViewBasis::historical(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        signal_basis,
        Some(wake),
    );

    assert_ne!(branch_head.kind(), historical.kind());
    assert_ne!(branch_head.identity(), historical.identity());
}

#[test]
fn temporal_basis_accepts_cdc_cursor_as_first_class_truth_family() {
    let admitted = admit(
        BridgeTemporalTruthViewBasis::cdc_cursor(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            BridgeTemporalCdcCursorIdentity::admit_bridge_owned("cursor-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            ClockDomain::MonotonicExecution,
            ClockTick::new(15),
            ClockAdvanceOrdinal::new(8),
            Some(ClockCheckpointId::new(10)),
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(14),
            WakeOrdinal::new(8),
            ClockTick::new(15),
        )),
    );

    assert_eq!(
        admitted.truth_basis().basis().native_truth_locator(),
        "cursor-a"
    );
}

#[test]
fn temporal_basis_equality_ignores_host_wall_clock_noise() {
    let time = std::sync::Arc::new(worth_execution::time::ControlledTime::at_unix_secs(10));
    let sampler = worth_execution::time::TimeSampler::new(time.clone());
    let wall_before = sampler.wall().unwrap();
    let boot_before = sampler.monotonic();
    let admitted = admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            ClockDomain::MonotonicExecution,
            ClockTick::new(21),
            ClockAdvanceOrdinal::new(12),
            Some(ClockCheckpointId::new(14)),
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(15),
            WakeOrdinal::new(10),
            ClockTick::new(21),
        )),
    );
    time.step_wall_back(std::time::Duration::from_secs(2));
    time.reboot_epoch();
    assert_ne!(sampler.wall().unwrap(), wall_before);
    assert!(sampler.monotonic().cmp_same_epoch(boot_before).is_err());
    let same = admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            ClockDomain::MonotonicExecution,
            ClockTick::new(21),
            ClockAdvanceOrdinal::new(12),
            Some(ClockCheckpointId::new(14)),
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(15),
            WakeOrdinal::new(10),
            ClockTick::new(21),
        )),
    );

    assert_eq!(admitted, same);
}

#[test]
fn temporal_basis_denies_empty_identity_fields_before_merged_construction() {
    let outcome = AdmittedBridgeTemporalBasis::admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::malformed_empty_truth_branch_for_validation(),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::malformed_empty_truth_branch_for_validation(),
            ClockDomain::MonotonicExecution,
            ClockTick::new(21),
            ClockAdvanceOrdinal::new(12),
            None,
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(16),
            WakeOrdinal::new(10),
            ClockTick::new(21),
        )),
    );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::TruthBasisDenied(_))
    ));
}

#[test]
fn temporal_basis_denies_metadata_only_clock_domains() {
    let outcome = AdmittedBridgeTemporalBasis::admit(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        BridgeTemporalSignalBasis::new(
            crate::truth_identity_fixtures::truth_branch_fixture("branch-a"),
            ClockDomain::WallClock,
            ClockTick::new(21),
            ClockAdvanceOrdinal::new(12),
            None,
        ),
        Some(BridgeTemporalWakeEvidence::new(
            TemporalWakeId::new(17),
            WakeOrdinal::new(11),
            ClockTick::new(21),
        )),
    );

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::SignalBasisDenied(_))
    ));
}

fn admit(
    truth_basis: BridgeTemporalTruthViewBasis,
    signal_basis: BridgeTemporalSignalBasis,
    wake: Option<BridgeTemporalWakeEvidence>,
) -> AdmittedBridgeTemporalBasis {
    match AdmittedBridgeTemporalBasis::admit(truth_basis, signal_basis, wake) {
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted temporal basis, got {outcome:?}"),
    }
}
