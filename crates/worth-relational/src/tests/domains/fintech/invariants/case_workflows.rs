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
    let bundle = publication
        .latest_bundle()
        .expect("publication bundle should exist after hostile workflow");
    let patch = publication
        .latest_patch()
        .expect("latest patch should exist after hostile workflow");
    let replay = publication
        .latest_replay()
        .expect("latest replay should exist after hostile workflow");
    let history = world.runtime.history();
    let commit = history
        .latest_commit()
        .expect("latest commit should exist after hostile workflow");
    let diagnostics = world.runtime.publication().diagnostics();

    assert_eq!(bundle.commit, commit);
    assert_eq!(bundle.patch, patch);
    assert_eq!(bundle.replay, replay);
    assert!(diagnostics
        .artifacts()
        .iter()
        .any(|artifact| artifact == &bundle.diagnostics_summary));
}

pub(crate) fn assert_merge_metadata_preserved(
    merge_parent_branches: &[BranchId],
    merge_base_count: usize,
    parent_count: usize,
    merge_branch: &BranchId,
    expected_parents: usize,
) {
    assert_eq!(merge_parent_branches, std::slice::from_ref(merge_branch));
    assert_eq!(merge_base_count, 1);
    assert_eq!(parent_count, expected_parents);
}
