//! A published prepared merge survives durable append, checkpoint and recovery unchanged.
use super::commit_meaning::assert_durable_commit_meaning_eq;
use super::merge_execution_observation::{capture_aspect_truth_bundle, diagnostic_field};
use super::merge_execution_recovery::checkpoint_and_recover_durable;
use super::merge_feature_execution::execute_feature_into_main_merge;
use worth_relational::facade::diagnostics::{DiagnosticCode, RelationalDiagnosticValue};
use worth_testkit_relational::durable_publication::durable_runtime;

#[test]
fn execute_prepared_merge_survives_durability_append_and_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (medium, runtime, merge, _main_head_commit_id, _feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let before_bundle = capture_aspect_truth_bundle(execution, &runtime, &[]);
    let merge_envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("live merge envelope");

    let (_recovery, recovered) =
        checkpoint_and_recover_durable(execution, &runtime, &medium, durable_runtime(&medium));
    let recovered_bundle = capture_aspect_truth_bundle(execution, &recovered, &[]);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, merge.commit.commit.commit_id)
        .unwrap()
        .expect("recovered merge envelope");

    assert_eq!(before_bundle.visible_truth, recovered_bundle.visible_truth);
    assert_durable_commit_meaning_eq(&merge_envelope, &recovered_envelope);
    assert!(merge_envelope
        .diagnostics_summary
        .entries
        .iter()
        .any(|entry| entry.code == DiagnosticCode::MergeExecutionPublished));
    let merge_execution_entry = merge_envelope
        .diagnostics_summary
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
        .expect("merge execution summary entry");
    assert_eq!(
        diagnostic_field(merge_execution_entry, "commit_id"),
        &RelationalDiagnosticValue::CommitId(merge.commit.commit.commit_id)
    );
    assert_eq!(
        diagnostic_field(merge_execution_entry, "execution_digest"),
        &RelationalDiagnosticValue::String(merge.execution_summary.execution_digest.clone())
    );
    assert_eq!(
        diagnostic_field(merge_execution_entry, "diagnostics_digest"),
        &RelationalDiagnosticValue::String(merge.execution_summary.diagnostics_digest.clone())
    );
}
