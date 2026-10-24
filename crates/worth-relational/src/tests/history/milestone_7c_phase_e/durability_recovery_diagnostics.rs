use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::facade::diagnostics::DiagnosticCode;
use crate::tests::support::{
    capture_aspect_truth_bundle, checkpoint_and_recover_with, diagnostic_field,
    durable_runtime_with_test_schema,
};

use super::fixtures::execute_feature_into_main_merge;

#[test]
fn execute_prepared_merge_survives_durability_append_and_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, merge, _main_head_commit_id, _feature_head_commit_id) =
        execute_feature_into_main_merge(execution);
    let before_bundle = capture_aspect_truth_bundle(execution, &runtime, &[], &[], &[]);
    let merge_envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("live merge envelope");

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, durable_runtime_with_test_schema);
    let recovered_bundle = capture_aspect_truth_bundle(execution, &recovered, &[], &[], &[]);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("recovered merge envelope");

    assert_eq!(before_bundle.visible_truth, recovered_bundle.visible_truth);
    assert_eq!(merge_envelope, recovered_envelope);
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
