use super::*;

pub(super) fn run_replacement_strategy_certification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ReplacementCertificationBundle {
    let root_path = unique_test_store_path("worth-relational-strategy-replacement-cert");
    let recovered_root = root_path.clone();
    let runtime = persisted_replacement_strategy_runtime(root_path);
    let replacement_entity = create_entity(execution, &runtime, "replace-target");
    let replacement_start_lineage = runtime
        .lineage_access()
        .for_record(replacement_entity)
        .expect("replacement entity lineage before strategy")
        .lineage_id;
    let replacement_commit = execute_strategy_commit(
        execution,
        &runtime,
        EntityReplacementReconciliationInput {
            entity_id: replacement_entity,
            replacement_client_key: "replace-target-v2".to_string(),
            desired_aspect_fields: strategy_name_and_replicas_patch("replace-main", 2),
        }
        .into_native_canonical_request(crate::facade::commit_strategies::StrategyCallerProvenance {
            request_origin: crate::facade::commit_strategies::StrategyRequestOrigin::Test,
            actor_identity: None,
            correlation_id: None,
        })
        .expect("native canonical strategy request"),
        None,
    );
    let current = runtime
        .read_truth()
        .read_version(execution, runtime.current_version_id());
    let replacement_record = changed_entities(&replacement_commit)
        .into_iter()
        .find_map(|entity_id| current.get_entity(entity_id).map(|record| record.entity_id))
        .expect("replacement entity visible after strategy");
    let replacement_end_lineage = runtime
        .lineage_access()
        .for_record(replacement_record)
        .expect("replacement entity lineage after strategy")
        .lineage_id;
    assert_ne!(replacement_start_lineage, replacement_end_lineage);
    let replacement_envelope = runtime
        .replay()
        .canonical_commit_envelope(replacement_commit.commit.commit_id)
        .expect("replacement envelope");
    assert!(replacement_envelope.lineage_decision_log().iter().any(
        |decision| decision.kind == crate::lineage::data::LineageDecisionKind::ReplaceAccepted
    ));
    let replacement_replay = runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: replacement_commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert!(
        replacement_replay.failure.is_none(),
        "replacement replay failed: {replacement_replay:?}"
    );
    assert!(replacement_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    let replacement_strategy_artifacts = replacement_commit
        .publication()
        .strategy_artifacts
        .as_ref()
        .expect("replacement strategy artifacts");
    let live_bundle = ReplacementCertificationBundle {
        replacement_commit_strategy_artifacts: replacement_strategy_artifacts.clone(),
        replacement_replay,
        replacement_lineage: ReplacementLineageEvidence {
            start_lineage: replacement_start_lineage,
            end_lineage: replacement_end_lineage,
            lineage_basis: replacement_envelope.lineage_digest_basis().clone(),
            event_batch_basis: replacement_envelope.event_batch_digest_basis().clone(),
            decision_log_basis: replacement_envelope.decision_log_digest_basis().clone(),
            normalized_client_key_count: replacement_strategy_artifacts
                .lowering_summary()
                .normalized_client_key_count(),
            lineage_transition_count: replacement_strategy_artifacts
                .lowering_summary()
                .lineage_transition_count(),
        },
    };
    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        persisted_replacement_strategy_runtime(recovered_root)
    });
    let recovered_replacement_envelope = recovered
        .replay()
        .canonical_commit_envelope(replacement_commit.commit.commit_id)
        .expect("recovered replacement envelope");
    let recovered_replacement_replay = recovered.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: replacement_commit.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
        },
        execution,
    );
    assert!(
        recovered_replacement_replay.failure.is_none(),
        "recovered replacement replay failed: {recovered_replacement_replay:?}"
    );
    let recovered_replacement_lineage = recovered
        .lineage_access()
        .for_record(replacement_record)
        .expect("recovered replacement entity lineage")
        .lineage_id;
    let recovered_replacement_strategy_artifacts = recovered_replacement_envelope
        .strategy_artifacts
        .as_ref()
        .expect("recovered replacement strategy artifacts");
    let recovered_bundle = ReplacementCertificationBundle {
        replacement_commit_strategy_artifacts: recovered_replacement_strategy_artifacts.clone(),
        replacement_replay: recovered_replacement_replay,
        replacement_lineage: ReplacementLineageEvidence {
            start_lineage: replacement_start_lineage,
            end_lineage: recovered_replacement_lineage,
            lineage_basis: recovered_replacement_envelope
                .lineage_digest_basis()
                .clone(),
            event_batch_basis: recovered_replacement_envelope
                .event_batch_digest_basis()
                .clone(),
            decision_log_basis: recovered_replacement_envelope
                .decision_log_digest_basis()
                .clone(),
            normalized_client_key_count: recovered_replacement_strategy_artifacts
                .lowering_summary()
                .normalized_client_key_count(),
            lineage_transition_count: recovered_replacement_strategy_artifacts
                .lowering_summary()
                .lineage_transition_count(),
        },
    };
    assert_eq!(recovered_bundle, live_bundle);
    live_bundle
}
