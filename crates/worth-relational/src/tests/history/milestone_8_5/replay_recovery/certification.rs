use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

pub(crate) fn certify_replay_recovery(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: StrategyReplayRecoveryInput,
) -> StrategyCertificationBundle {
    let StrategyReplayRecoveryInput {
        runtime,
        recovered_root,
        entity,
        feature_branch,
        aspect_overlap_branch,
        aspect_disjoint_branch,
        controller_sequence_branch,
        main_commit,
        feature_commit,
        controller_feature_idempotent_commit,
        replacement_certification,
    } = input;
    let planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                feature_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("final merge planning");
    let aspect_overlap_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_overlap_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("final aspect overlap merge planning");
    let aspect_disjoint_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_disjoint_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("final aspect disjoint merge planning");
    let controller_sequence_planning = runtime
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                controller_sequence_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("final controller sequence merge planning");
    let main_replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: main_commit.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    let feature_replay = runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: feature_commit.commit.commit_id,
                branch_id: feature_branch.clone(),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(
        main_replay.failure.is_none(),
        "main replay failed: {main_replay:?}"
    );
    assert!(main_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(feature_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    let current = runtime
        .read_truth()
        .read_version(execution, runtime.current_version_id())
        .expect("read request remains live");
    let live_branch_heads = StrategyBranchHeadEvidence {
        main: runtime.history().branch_head(&BranchId("main".to_string())),
        feature: runtime.history().branch_head(&feature_branch),
    };
    let live_visible_truth = StrategyVisibleTruthEvidence {
        entity_name: read_entity_name(current.get_entity(entity).expect("entity visible")),
        branch_heads: live_branch_heads.clone(),
    };

    let mut live_bundle = StrategyCertificationBundle {
        main_commit_strategy_artifacts: main_commit
            .publication()
            .strategy_artifacts
            .as_ref()
            .expect("main strategy artifacts")
            .clone(),
        feature_commit_strategy_artifacts: feature_commit
            .publication()
            .strategy_artifacts
            .as_ref()
            .expect("feature strategy artifacts")
            .clone(),
        replacement: replacement_certification.clone(),
        merge_conflict: planning.digest_basis.conflict.clone(),
        merge_lowered_plan: planning.digest_basis.lowered_plan.clone(),
        aspect_overlap_merge_conflict: aspect_overlap_planning.digest_basis.conflict.clone(),
        aspect_overlap_merge_lowered_plan: aspect_overlap_planning
            .digest_basis
            .lowered_plan
            .clone(),
        aspect_disjoint_merge_conflict: aspect_disjoint_planning.digest_basis.conflict.clone(),
        aspect_disjoint_merge_lowered_plan: aspect_disjoint_planning
            .digest_basis
            .lowered_plan
            .clone(),
        controller_sequence_merge_conflict: controller_sequence_planning
            .digest_basis
            .conflict
            .clone(),
        controller_sequence_merge_lowered_plan: controller_sequence_planning
            .digest_basis
            .lowered_plan
            .clone(),
        main_replay,
        feature_replay,
        controller_sequence_noop: ControllerSequenceNoopEvidence {
            strategy_artifacts: controller_feature_idempotent_commit
                .publication()
                .strategy_artifacts
                .as_ref()
                .expect("controller idempotent strategy artifacts")
                .clone(),
            changed_record_count: controller_feature_idempotent_commit
                .change_summary()
                .expect("controller idempotent change summary")
                .changed_record_count,
            patch_record_count: controller_feature_idempotent_commit
                .publication_summary()
                .expect("controller idempotent publication summary")
                .patch_record_count,
        },
        missing_executor_replay: StrategyReplayMismatchEvidence {
            strategy_surface_mismatch_present: false,
        },
        failing_executor_replay: StrategyReplayMismatchEvidence {
            strategy_surface_mismatch_present: false,
        },
        branch_heads: live_branch_heads,
        visible_truth: live_visible_truth,
    };

    let recovery_plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::AuditRecoveryVerification,
    )
    .unwrap();

    let (missing_executor_mismatch_present, failing_executor_mismatch_present) =
        executor_faults::certify(
            execution,
            &recovery_plan,
            &recovered_root,
            main_commit.commit.commit_id,
        );
    live_bundle.missing_executor_replay = StrategyReplayMismatchEvidence {
        strategy_surface_mismatch_present: missing_executor_mismatch_present,
    };
    live_bundle.failing_executor_replay = StrategyReplayMismatchEvidence {
        strategy_surface_mismatch_present: failing_executor_mismatch_present,
    };

    let (_recovery, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        persisted_strategy_runtime(recovered_root)
    });

    let recovered_planning = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                feature_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("recovered merge planning");
    let recovered_main_replay = recovered
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: main_commit.commit.commit_id,
                branch_id: BranchId("main".to_string()),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    let recovered_feature_replay = recovered
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: feature_commit.commit.commit_id,
                branch_id: feature_branch.clone(),
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert!(
        recovered_main_replay.failure.is_none(),
        "recovered main replay failed: {recovered_main_replay:?}"
    );
    assert!(
        recovered_feature_replay.failure.is_none(),
        "recovered feature replay failed: {recovered_feature_replay:?}"
    );
    assert!(recovered_main_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    assert!(recovered_feature_replay
        .compared_surfaces
        .contains(&ReplayObservableSurface::Strategy));
    let recovered_aspect_overlap_planning = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_overlap_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("recovered aspect overlap planning");
    let recovered_aspect_disjoint_planning = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                aspect_disjoint_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("recovered aspect disjoint planning");
    let recovered_controller_sequence_planning = recovered
        .merge()
        .inspect_planning_scope(
            execution,
            MergePlanningRequest::new(
                BranchId("main".to_string()),
                controller_sequence_branch.clone(),
                MergeIntent::ReconcileIntoTarget,
            ),
        )
        .expect("recovered controller sequence planning");
    let recovered_main_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, main_commit.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .expect("recovered main envelope");
    let recovered_feature_envelope = recovered
        .replay()
        .canonical_commit_envelope(execution, feature_commit.commit.commit_id)
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .expect("recovered feature envelope");
    let recovered_current = recovered
        .read_truth()
        .read_version(execution, recovered.current_version_id())
        .expect("read request remains live");
    let recovered_controller_noop_envelope = recovered
        .replay()
        .canonical_commit_envelope(
            execution,
            controller_feature_idempotent_commit.commit.commit_id,
        )
        .unwrap()
        .map(|envelope| envelope.as_ref().clone())
        .expect("recovered controller noop envelope");
    let recovered_branch_heads = StrategyBranchHeadEvidence {
        main: recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        feature: recovered.history().branch_head(&feature_branch),
    };
    let recovered_visible_truth = StrategyVisibleTruthEvidence {
        entity_name: read_entity_name(
            recovered_current
                .get_entity(entity)
                .expect("recovered entity visible"),
        ),
        branch_heads: recovered_branch_heads.clone(),
    };
    let recovered_bundle = StrategyCertificationBundle {
        main_commit_strategy_artifacts: recovered_main_envelope
            .strategy_artifacts
            .as_ref()
            .expect("recovered main strategy artifacts")
            .clone(),
        feature_commit_strategy_artifacts: recovered_feature_envelope
            .strategy_artifacts
            .as_ref()
            .expect("recovered feature strategy artifacts")
            .clone(),
        replacement: replacement_certification,
        merge_conflict: recovered_planning.digest_basis.conflict.clone(),
        merge_lowered_plan: recovered_planning.digest_basis.lowered_plan.clone(),
        aspect_overlap_merge_conflict: recovered_aspect_overlap_planning
            .digest_basis
            .conflict
            .clone(),
        aspect_overlap_merge_lowered_plan: recovered_aspect_overlap_planning
            .digest_basis
            .lowered_plan
            .clone(),
        aspect_disjoint_merge_conflict: recovered_aspect_disjoint_planning
            .digest_basis
            .conflict
            .clone(),
        aspect_disjoint_merge_lowered_plan: recovered_aspect_disjoint_planning
            .digest_basis
            .lowered_plan
            .clone(),
        controller_sequence_merge_conflict: recovered_controller_sequence_planning
            .digest_basis
            .conflict
            .clone(),
        controller_sequence_merge_lowered_plan: recovered_controller_sequence_planning
            .digest_basis
            .lowered_plan
            .clone(),
        main_replay: recovered_main_replay,
        feature_replay: recovered_feature_replay,
        controller_sequence_noop: ControllerSequenceNoopEvidence {
            strategy_artifacts: recovered_controller_noop_envelope
                .strategy_artifacts
                .as_ref()
                .expect("recovered controller noop strategy artifacts")
                .clone(),
            changed_record_count: recovered_controller_noop_envelope
                .patch
                .authoritative_record_patches
                .len(),
            patch_record_count: recovered_controller_noop_envelope
                .patch
                .authoritative_record_patches
                .len(),
        },
        missing_executor_replay: StrategyReplayMismatchEvidence {
            strategy_surface_mismatch_present: missing_executor_mismatch_present,
        },
        failing_executor_replay: StrategyReplayMismatchEvidence {
            strategy_surface_mismatch_present: failing_executor_mismatch_present,
        },
        branch_heads: recovered_branch_heads,
        visible_truth: recovered_visible_truth,
    };
    assert_eq!(recovered_bundle, live_bundle);
    live_bundle
}

#[path = "certification/executor_faults.rs"]
mod executor_faults;
