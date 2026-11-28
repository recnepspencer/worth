use crate::facade::history::BranchId;

use super::super::comparisons::{
    compare_case_truth, compare_observability_overlap, compare_recovery_probe,
};
use super::super::fixture::FintechWorld;
use super::super::probes::{CaseTruthProbe, ObservabilityProbe, RecoveryProbe, ReplayProbe};

pub(crate) fn assert_correction_case_transition(
    baseline: &CaseTruthProbe,
    post_mutation: &CaseTruthProbe,
) {
    let mismatches = compare_case_truth(baseline, post_mutation);
    assert!(
        mismatches.contains(&"corrected_trade_count".to_string()),
        "late trade correction should change corrected_trade_count, saw {mismatches:?}"
    );
    assert!(
        post_mutation.audit_record_count > 0,
        "late trade correction should expose at least one audit record"
    );
}

pub(crate) fn assert_intraday_risk_case_transition(
    baseline: &CaseTruthProbe,
    post_mutation: &CaseTruthProbe,
) {
    let mismatches = compare_case_truth(baseline, post_mutation);
    assert!(
        mismatches.contains(&"aspect_state_fingerprints".to_string()),
        "intraday risk should change the case aspect state fingerprints, saw {mismatches:?}"
    );
    assert!(
        post_mutation.open_breach_count > 0,
        "intraday risk workflow should expose an open breach"
    );
}

pub(crate) fn assert_settlement_repair_case_transition(
    baseline: &CaseTruthProbe,
    post_mutation: &CaseTruthProbe,
) {
    let mismatches = compare_case_truth(baseline, post_mutation);
    assert!(
        mismatches.contains(&"repaired_settlement_count".to_string()),
        "settlement repair should change repaired_settlement_count, saw {mismatches:?}"
    );
    assert!(
        post_mutation.audit_record_count > 0,
        "settlement repair should expose at least one audit record"
    );
}

pub(crate) fn assert_observability_overlap_stable(
    baseline: &ObservabilityProbe,
    post_mutation: &ObservabilityProbe,
) {
    let mismatches = compare_observability_overlap(baseline, post_mutation);
    assert!(
        mismatches.is_empty(),
        "observability overlap should remain stable, saw {mismatches:?}"
    );
}

pub(crate) fn assert_replay_targets_branch(replay: &ReplayProbe, branch_id: &BranchId) {
    assert_eq!(replay.branch_name, branch_id.0);
}

pub(crate) fn assert_recovery_matches_truth(expected: &RecoveryProbe, recovered: &RecoveryProbe) {
    let mismatches = compare_recovery_probe(expected, recovered);
    assert!(
        mismatches.is_empty(),
        "recovered truth should match expected branch heads and commit state, saw {mismatches:?}"
    );
}

pub(crate) fn assert_snapshot_release_contract(
    baseline: &CaseTruthProbe,
    historical_after_mutation: &CaseTruthProbe,
    historical_after_release: &CaseTruthProbe,
) {
    assert!(
        compare_case_truth(baseline, historical_after_mutation).is_empty(),
        "historical snapshot truth should remain stable after later mutation"
    );
    assert!(
        compare_case_truth(baseline, historical_after_release).is_empty(),
        "historical version read should remain stable after snapshot release"
    );
}

pub(crate) fn assert_observability_surfaces_agree(world: &FintechWorld) {
    let publication = world.runtime.publication();
    let observation = publication.observation_snapshot();
    let history = world.runtime.history();
    assert_eq!(observation.latest_patch_position, history.latest_patch_stream_position());
    assert_eq!(observation.latest_commit_id, history.latest_commit().map(|commit| commit.commit_id));
}
