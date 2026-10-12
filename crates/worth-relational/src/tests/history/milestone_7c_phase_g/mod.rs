mod artifacts;
mod certification_cases;
mod recovery_certification;
mod schema_fixtures;
mod stale_prepared_merge_denials;

use artifacts::{
    AuthoritativeMergeExecutionCertificationSuite, MergeExecutionCertificationArtifacts,
};
use certification_cases::{
    certify_exact_shared_merge_execution, certify_prefer_richer_merge_execution,
    certify_source_only_addition_merge_execution,
};

#[test]
fn authoritative_merge_execution_certification_emits_machine_checkable_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = AuthoritativeMergeExecutionCertificationSuite {
        exact_shared: certify_exact_shared_merge_execution(execution),
        source_only_addition: certify_source_only_addition_merge_execution(execution),
        prefer_richer_reconcile: certify_prefer_richer_merge_execution(execution),
    };

    for certification in [
        &suite.exact_shared,
        &suite.source_only_addition,
        &suite.prefer_richer_reconcile,
    ] {
        assert_machine_checkable_merge_execution_artifact(certification);
    }
}

fn assert_machine_checkable_merge_execution_artifact(
    certification: &MergeExecutionCertificationArtifacts,
) {
    assert!(certification.merge_execution_digest.len() > 8);
    assert!(certification.merge_execution_diagnostics_digest.len() > 8);
    assert!(certification.visible_entity_count > 0);
    assert_eq!(certification.visible_relation_count, 0);
    assert!(certification.replay_verified);
    assert!(certification.recovery_envelope_matches);
    assert!(certification.recovery_truth_matches);
    assert!(certification.branch_heads_match);
}
