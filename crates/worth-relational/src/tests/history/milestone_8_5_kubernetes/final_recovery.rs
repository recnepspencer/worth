use super::*;

pub(super) struct FinalRecoveryCertificationInput {
    pub(super) runtime: RelationalRuntime,
    pub(super) recovered_root: std::sync::Arc<crate::tests::support::TestDurableMedium>,
    pub(super) entity: crate::facade::identity::EntityId,
    pub(super) main_branch: BranchId,
    pub(super) controller_branch: BranchId,
    pub(super) overlap_main_head: crate::history::data::RelationalCommitReceipt,
    pub(super) overlap_controller_head: crate::history::data::RelationalCommitReceipt,
    pub(super) narrowed_main_head: crate::history::data::RelationalCommitReceipt,
    pub(super) narrowed_controller_head: crate::history::data::RelationalCommitReceipt,
    pub(super) rebroadened_main_head: crate::history::data::RelationalCommitReceipt,
    pub(super) rebroadened_controller_head: crate::history::data::RelationalCommitReceipt,
    pub(super) broad_intent_commit: CommitResult,
    pub(super) first_converge_commit: CommitResult,
    pub(super) rebroadened_intent_commit: CommitResult,
    pub(super) revalidation_commit: CommitResult,
    pub(super) live_bundle: KubernetesIntentCertificationBundle,
}

pub(super) fn certify_final_recovery(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: FinalRecoveryCertificationInput,
) -> KubernetesIntentCertificationBundle {
    let FinalRecoveryCertificationInput {
        runtime,
        recovered_root,
        entity,
        main_branch,
        controller_branch,
        overlap_main_head,
        overlap_controller_head,
        narrowed_main_head,
        narrowed_controller_head,
        rebroadened_main_head,
        rebroadened_controller_head,
        broad_intent_commit,
        first_converge_commit,
        rebroadened_intent_commit,
        revalidation_commit,
        live_bundle,
    } = input;

    let recovered = recover_stage(execution, &runtime, recovered_root);
    let recovered_overlap_from_final = recover_stage_from_final_history(
        execution,
        &recovered,
        crate::tests::support::TestDurableMedium::new(),
        overlap_controller_head,
        overlap_main_head,
    );
    let recovered_overlap_planning = planning_for(
        execution,
        &recovered_overlap_from_final,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(
        &recovered_overlap_planning,
        entity,
        "final-history recovered overlap",
    );
    assert_eq!(
        planning_evidence(&recovered_overlap_planning),
        live_bundle.overlap_conflict
    );
    let recovered_narrowed_from_final = recover_stage_from_final_history(
        execution,
        &recovered,
        crate::tests::support::TestDurableMedium::new(),
        narrowed_controller_head,
        narrowed_main_head,
    );
    let recovered_narrowed_planning = planning_for(
        execution,
        &recovered_narrowed_from_final,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_non_strategy_conflict(
        &recovered_narrowed_planning,
        entity,
        "final-history recovered narrowed",
    );
    assert_eq!(
        planning_evidence(&recovered_narrowed_planning),
        live_bundle.narrowed_non_conflict
    );
    let recovered_rebroadened_from_final = recover_stage_from_final_history(
        execution,
        &recovered,
        crate::tests::support::TestDurableMedium::new(),
        rebroadened_controller_head,
        rebroadened_main_head,
    );
    let recovered_rebroadened_planning = planning_for(
        execution,
        &recovered_rebroadened_from_final,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_strategy_conflict(
        &recovered_rebroadened_planning,
        entity,
        "final-history recovered rebroadened",
    );
    assert_eq!(
        planning_evidence(&recovered_rebroadened_planning),
        live_bundle.rebroadened_conflict
    );
    let recovered_revalidated_planning = planning_for(
        execution,
        &recovered,
        controller_branch.clone(),
        main_branch.clone(),
    );
    assert_converged_strategy_overlap(
        &recovered_revalidated_planning,
        entity,
        "recovered revalidated shared truth",
    );

    let recovered_broad_intent_replay = replay_commit(
        execution,
        &recovered,
        broad_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    let recovered_first_converge_replay = replay_commit(
        execution,
        &recovered,
        first_converge_commit.commit.commit_id,
        controller_branch.clone(),
    );
    let recovered_rebroadened_intent_replay = replay_commit(
        execution,
        &recovered,
        rebroadened_intent_commit.commit.commit_id,
        main_branch.clone(),
    );
    let recovered_revalidation_replay = replay_commit(
        execution,
        &recovered,
        revalidation_commit.commit.commit_id,
        controller_branch.clone(),
    );
    assert_strategy_replay_clean(&recovered_broad_intent_replay, "recovered broad intent");
    assert_strategy_replay_clean(&recovered_first_converge_replay, "recovered first converge");
    assert_strategy_replay_clean(
        &recovered_rebroadened_intent_replay,
        "recovered rebroadened intent",
    );
    assert_eq!(
        planning_evidence(&recovered_revalidated_planning),
        live_bundle.revalidated_converged_overlap
    );
    let recovered_revalidation_envelope = recovered
        .replay()
        .canonical_commit_envelope(revalidation_commit.commit.commit_id)
        .expect("recovered revalidation envelope");
    assert_eq!(
        KubernetesNoopEvidence {
            strategy_artifacts: recovered_revalidation_envelope
                .strategy_artifacts
                .as_ref()
                .expect("recovered revalidation strategy artifacts")
                .clone(),
            changed_record_count: recovered_revalidation_envelope
                .patch
                .authoritative_record_patches
                .len(),
            patch_record_count: recovered_revalidation_envelope
                .patch
                .authoritative_record_patches
                .len(),
        },
        live_bundle.revalidation_noop
    );
    assert_eq!(
        recovered_broad_intent_replay,
        live_bundle.broad_intent_replay
    );
    assert_eq!(
        recovered_first_converge_replay,
        live_bundle.first_converge_replay
    );
    assert_eq!(
        recovered_rebroadened_intent_replay,
        live_bundle.rebroadened_intent_replay
    );
    assert_eq!(
        recovered_revalidation_replay,
        live_bundle.revalidation_noop_replay
    );
    assert_strategy_replay_clean(
        &recovered_revalidation_replay,
        "recovered revalidation converge",
    );
    assert_eq!(
        KubernetesBranchHeadEvidence {
            main: recovered.history().branch_head(&main_branch),
            controller: recovered.history().branch_head(&controller_branch),
        },
        live_bundle.branch_heads
    );
    assert_eq!(
        KubernetesBranchVisibleTruthEvidence {
            main: visible_truth_for_branch(execution, &recovered, &main_branch, entity),
            controller: visible_truth_for_branch(execution, &recovered, &controller_branch, entity),
        },
        live_bundle.visible_truth
    );
    live_bundle
}
