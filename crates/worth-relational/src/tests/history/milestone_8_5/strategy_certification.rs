use super::*;

pub(super) fn run_strategy_merge_certification(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> StrategyCertificationBundle {
    let root_path = unique_test_store_path("worth-relational-strategy-cert");
    let recovered_root = root_path.clone();
    let runtime = persisted_strategy_runtime(root_path);
    let entity = create_entity(execution, &runtime, "service");
    let aspect_overlap_entity = create_entity(execution, &runtime, "aspect-overlap");
    let aspect_disjoint_entity = create_entity(execution, &runtime, "aspect-disjoint");
    let feature_branch = create_branch_from_main(&runtime, "strategy-feature");
    let aspect_overlap_branch = create_branch_from_main(&runtime, "aspect-overlap-feature");
    let aspect_disjoint_branch = create_branch_from_main(&runtime, "aspect-disjoint-feature");

    let primary_conflict = merge_strategy_conflict::certify_primary_strategy_conflict(
        execution,
        &runtime,
        entity,
        &feature_branch,
    );
    merge_aspect_conflicts::certify_overlapping_aspect_strategy_conflict(
        execution,
        &runtime,
        aspect_overlap_entity,
        &aspect_overlap_branch,
    );
    merge_aspect_conflicts::certify_disjoint_aspect_strategy_truth(
        execution,
        &runtime,
        aspect_disjoint_entity,
        &aspect_disjoint_branch,
    );
    let controller_sequence =
        controller_sequence::certify_controller_sequence_shared_truth(execution, &runtime);

    let main_commit = primary_conflict.main_commit;
    let feature_commit = primary_conflict.feature_commit;
    let controller_sequence_branch = controller_sequence.branch;
    let controller_feature_idempotent_commit = controller_sequence.idempotent_commit;

    let replacement_certification = run_replacement_strategy_certification(execution);

    replay_recovery::certify_replay_recovery(
        execution,
        replay_recovery::StrategyReplayRecoveryInput {
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
        },
    )
}
