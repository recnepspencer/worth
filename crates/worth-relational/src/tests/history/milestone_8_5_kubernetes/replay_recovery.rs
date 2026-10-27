use super::*;

pub(super) fn replay_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit_id: crate::history::data::CommitId,
    branch_id: BranchId,
) -> crate::facade::replay::RelationalReplayOutcome {
    runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id,
                branch_id,
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::AuditRecoveryVerification,
            },
            execution,
        )
        .unwrap()
}

pub(super) fn assert_strategy_replay_clean(
    replay: &crate::facade::replay::RelationalReplayOutcome,
    stage: &str,
) {
    assert!(
        replay
            .compared_surfaces
            .contains(&ReplayObservableSurface::Strategy),
        "expected strategy replay surface during {stage}: {replay:?}"
    );
    assert!(
        replay
            .mismatches
            .iter()
            .all(|mismatch| mismatch.surface != ReplayObservableSurface::Strategy),
        "unexpected strategy replay mismatch during {stage}: {replay:?}"
    );
}

pub(super) fn planning_evidence(
    planning: &crate::merge::data::MergePlanningArtifactCore,
) -> KubernetesPlanningEvidence {
    KubernetesPlanningEvidence {
        conflict: KubernetesConflictEvidence {
            records: planning.digest_basis.conflict.records.clone(),
            classes: planning.digest_basis.conflict.classes.clone(),
            validated_schema_correspondence: planning
                .digest_basis
                .conflict
                .validated_schema_correspondence
                .clone(),
            strategy_conflict_classes: planning
                .digest_basis
                .conflict
                .strategy_conflict_classes
                .clone(),
            source_strategy_descriptors: planning
                .digest_basis
                .conflict
                .source_strategy_descriptors
                .clone(),
            target_strategy_descriptors: planning
                .digest_basis
                .conflict
                .target_strategy_descriptors
                .clone(),
            relation_evidence: planning.digest_basis.conflict.relation_evidence.clone(),
            aspect_evidence_keys: planning.digest_basis.conflict.aspect_evidence_keys.clone(),
            aspect_evidence_comparisons: planning
                .digest_basis
                .conflict
                .aspect_evidence_comparisons
                .clone(),
        },
        lowered_plan: planning.digest_basis.lowered_plan.clone(),
        decision_log: planning.decision_log_digest_basis.clone(),
    }
}

pub(super) fn recover_stage(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    root_path: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    let (_recovery, recovered) =
        checkpoint_and_recover_with(execution, runtime, || persisted_strategy_runtime(root_path));
    recovered
}

pub(super) fn recover_stage_from_final_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &RelationalRuntime,
    root_path: std::sync::Arc<crate::tests::support::TestDurableMedium>,
    source_head: crate::history::data::RelationalCommitReceipt,
    target_head: crate::history::data::RelationalCommitReceipt,
) -> RelationalRuntime {
    let mut chain = source
        .history()
        .ancestor_closure_by_commit_id_order(source_head.commit_id)
        .into_iter()
        .chain(
            source
                .history()
                .ancestor_closure_by_commit_id_order(target_head.commit_id),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    chain.sort_unstable();
    let replay_access = source.replay();
    let checkpoint = source
        .durable_checkpoints()
        .into_iter()
        .rev()
        .find(|checkpoint| {
            checkpoint
                .coverage
                .up_to_commit
                .as_ref()
                .map(|commit| chain.contains(&commit.commit_id))
                .unwrap_or(false)
        })
        .map(|checkpoint| checkpoint.as_ref().clone());
    let tail_start = checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.coverage.up_to_commit.as_ref())
        .map(|commit| commit.commit_id);
    let tail_log = chain
        .iter()
        .copied()
        .filter(|commit_id| tail_start.is_none_or(|start| *commit_id > start))
        .filter_map(|commit_id| {
            let position = source.history().canonical_stream_position(commit_id)?;
            replay_access
                .canonical_commit_envelope(execution, commit_id)
                .unwrap()
                .map(|envelope| envelope.as_ref().clone())
                .map(|envelope| {
                    crate::history::data::PositionedCanonicalCommit::for_test(
                        position,
                        std::sync::Arc::new(envelope),
                    )
                })
        })
        .collect::<Vec<_>>();
    let restore_authoritative_envelope_commit_ids = tail_log
        .iter()
        .filter(|envelope| envelope.strategy_artifacts.is_some())
        .map(|envelope| envelope.commit.commit_id)
        .collect::<Vec<_>>();
    let descriptor_semantics_version = tail_log
        .last()
        .map(|envelope| envelope.descriptor_semantics_version)
        .unwrap_or_default();
    // The staged chain replays only some commits, so it carries the forks of
    // the branches those commits are on, as a partial replay chain does.
    let covered_transitions = checkpoint.as_ref().map_or(0, |checkpoint| {
        checkpoint.coverage.up_to_branch_transition.0
    });
    let mut chain_branches = tail_log
        .iter()
        .map(|envelope| envelope.branch_context.clone())
        .collect::<BTreeSet<_>>();
    let mut forks = source
        .durability()
        .reconstruct_branch_transition_history(
            execution,
            crate::branch::RelationalBranchTransitionSequence(covered_transitions),
        )
        .unwrap()
        .into_iter()
        .rev()
        .filter_map(|transition| {
            if let crate::branch::RelationalBranchTransition::Fork { target, source, .. } =
                transition.transition()
                && chain_branches.contains(target)
            {
                chain_branches.insert(source.clone());
                Some(transition)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    forks.reverse();
    let plan = crate::durability::data::RecoveryPlan::new(
        source.config().clone(),
        checkpoint,
        tail_log
            .into_iter()
            .map(crate::durability::migration::ReadmittedCanonicalCommit::exact)
            .collect(),
        crate::durability::data::RecoveryBranchTransitions::Selected(forks),
        crate::durability::data::RecoveryAuthorityContinuityCheck::verified_at(
            crate::replay::data::ReplayVerificationLayer::DigestParity,
        ),
        crate::durability::data::RecoveryVerificationMode::AuditRecoveryVerification,
        descriptor_semantics_version,
        restore_authoritative_envelope_commit_ids,
        source.durability.cold_source(),
    )
    .with_commit_strategy_executors(source.commit_strategy_executor_registry().clone());
    let mut recovered = persisted_strategy_runtime(root_path);
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("recover staged runtime from final history");
    if let Some(base_commit_id) = recovered
        .history()
        .max_commit_id_common_ancestor(source_head.commit_id, target_head.commit_id)
    {
        let base_version_id = recovered
            .history()
            .demand_commit_envelope(execution, base_commit_id)
            .unwrap()
            .map(|envelope| envelope.commit.version_id);
        if let Some(base_version_id) = base_version_id {
            recovered
                .history_authority()
                .retain_version_for_replay(execution, base_version_id)
                .unwrap();
        }
    }
    recovered
}
