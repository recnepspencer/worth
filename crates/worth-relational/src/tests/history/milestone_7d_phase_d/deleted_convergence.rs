use super::*;

#[test]
fn deleted_on_both_sides_merge_commit_has_replay_and_recovery_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "shared");
    create_branch_from_main(&runtime, "feature");
    delete_entity(execution, &runtime, entity);
    delete_entity_on_branch(execution, &runtime, entity, BranchId("feature".to_string()));

    let prepared = runtime
        .prepare_merge_execution(
            execution,
            MergeExecutionRequest {
                target_branch: BranchId("main".to_string()),
                source_branch: BranchId("feature".to_string()),
                merge_intent: MergeIntent::ReconcileIntoTarget,
            },
        )
        .expect("prepared deleted-on-both-sides merge");
    let merge = runtime
        .execute_prepared_merge(execution, prepared)
        .expect("executed deleted-on-both-sides merge");

    assert_eq!(merge.structural_summary.executed_record_count, 1);
    assert_eq!(
        merge
            .structural_summary
            .converged_deleted_on_both_sides_count,
        1
    );
    assert_eq!(
        merge
            .structural_summary
            .deleted_on_both_sides_lineage_unchanged_count,
        1
    );
    assert_eq!(merge.structural_summary.emitted_mutation_intent_count, 0);

    let live_envelope = runtime
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("live merge envelope");
    let live_truth = capture_aspect_truth_bundle(execution, &runtime, &[entity], &[], &[]);

    let replay = runtime
        .replay_authority()
        .replay_commit(
            crate::facade::replay::RelationalReplayRequest {
                commit_id: merge.commit.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: crate::facade::replay::ReplayExecutionMode::SerialDeterministic,
                verification_mode:
                    crate::facade::replay::ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(
        replay.failure.is_none(),
        "replay certification failure: {replay:?}"
    );

    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, &runtime, persisted_runtime_with_test_schema);
    let recovered_envelope = recovered
        .replay()
        .canonical_commit_envelope(merge.commit.commit.commit_id)
        .expect("recovered merge envelope");
    let recovered_truth = capture_aspect_truth_bundle(execution, &recovered, &[entity], &[], &[]);

    assert_eq!(live_envelope, recovered_envelope);
    assert_eq!(live_truth.visible_truth, recovered_truth.visible_truth);
    assert_eq!(
        live_truth.entity_history_digests,
        recovered_truth.entity_history_digests
    );
    assert_eq!(
        live_envelope.diagnostics_summary,
        recovered_envelope.diagnostics_summary
    );

    let summary_entry = live_envelope
        .diagnostics_summary
        .entries
        .iter()
        .find(|entry| entry.code == DiagnosticCode::MergeExecutionPublished)
        .expect("merge execution summary entry");
    assert_eq!(
        diagnostic_field(summary_entry, "converged_deleted_on_both_sides_count"),
        &RelationalDiagnosticValue::Unsigned(1)
    );
    assert_eq!(
        diagnostic_field(
            summary_entry,
            "deleted_on_both_sides_lineage_unchanged_count"
        ),
        &RelationalDiagnosticValue::Unsigned(1)
    );
    assert_eq!(
        diagnostic_field(summary_entry, "execution_digest"),
        &RelationalDiagnosticValue::String(merge.execution_summary.execution_digest.clone())
    );
    assert_eq!(
        diagnostic_field(summary_entry, "diagnostics_digest"),
        &RelationalDiagnosticValue::String(merge.execution_summary.diagnostics_digest.clone())
    );

    let live_execution_artifact = runtime
        .publication()
        .diagnostics()
        .artifacts()
        .iter()
        .find(|artifact| {
            artifact.kind == crate::facade::diagnostics::DiagnosticsArtifactKind::DetailedTrace
                && artifact.entries.iter().any(|entry| {
                    entry.code == DiagnosticCode::MergeExecutionPublished
                        && diagnostic_field(entry, "commit_id")
                            == &RelationalDiagnosticValue::CommitId(merge.commit.commit.commit_id)
                })
        })
        .expect("live merge execution artifact")
        .clone();
    let record_entry = live_execution_artifact
        .entries
        .iter()
        .find(|entry| {
            diagnostic_field_optional(entry, "record_class")
                == Some(&RelationalDiagnosticValue::String(
                    "converge_deleted_on_both_sides".to_string(),
                ))
        })
        .expect("deleted-on-both-sides execution row");
    assert_eq!(
        diagnostic_field(record_entry, "lineage_continuity"),
        &RelationalDiagnosticValue::String("Unchanged".to_string())
    );
    assert!(matches!(
        diagnostic_field(record_entry, "equality_witness_digest"),
        RelationalDiagnosticValue::String(_)
    ));
}
