use super::ExactBoundarySymbol;

pub(super) const PLANNER_AND_EXECUTOR_BOUNDARIES: &[ExactBoundarySymbol] = &[
    boundary!(
        "evaluation plan construction entry",
        "logic/planner/planning/mod.rs",
        "../../../../../logic/planner/planning/mod.rs",
        "build_evaluation_plan_with_policy_resolver",
        2
    ),
    boundary!(
        "readiness prevalidation owner",
        "logic/planner/precompute/eligibility.rs",
        "../../../../../logic/planner/precompute/eligibility.rs",
        "prevalidate_stage_tasks",
        1
    ),
    boundary!(
        "epoch readiness call and import",
        "logic/planner/precompute/read_preparation.rs",
        "../../../../../logic/planner/precompute/read_preparation.rs",
        "prevalidate_stage_tasks",
        2
    ),
    boundary!(
        "prepared-plan execution facade",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_prepared_plan",
        1
    ),
    boundary!(
        "prepared-plan execution owner",
        "logic/planner/execution/prepared_plan.rs",
        "../../../../../logic/planner/execution/prepared_plan.rs",
        "execute_prepared_plan",
        1
    ),
    boundary!(
        "policy-governed execution entry",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_prepared_plan_with_policy",
        1
    ),
    boundary!(
        "prepared-plan policy delegation",
        "logic/planner/execution/prepared_plan.rs",
        "../../../../../logic/planner/execution/prepared_plan.rs",
        "execute_prepared_plan_with_policy",
        2
    ),
    boundary!(
        "temporal-readiness execution entry",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_prepared_plan_with_policy_and_temporal_lowering",
        2
    ),
    boundary!(
        "epoch stage dispatch call and import",
        "logic/planner/execution/epochs.rs",
        "../../../../../logic/planner/execution/epochs.rs",
        "execute_stage",
        2
    ),
    boundary!(
        "stage execution owner",
        "logic/planner/execution/stage.rs",
        "../../../../../logic/planner/execution/stage.rs",
        "execute_stage",
        1
    ),
    boundary!(
        "stage precompute orchestration call",
        "logic/planner/execution/stage.rs",
        "../../../../../logic/planner/execution/stage.rs",
        "perform_stage_precompute",
        2
    ),
    boundary!(
        "stage precompute owner",
        "logic/planner/precompute/stage.rs",
        "../../../../../logic/planner/precompute/stage.rs",
        "perform_stage_precompute",
        1
    ),
    boundary!(
        "stage application orchestration call",
        "logic/planner/execution/stage.rs",
        "../../../../../logic/planner/execution/stage.rs",
        "apply_stage",
        2
    ),
    boundary!(
        "stage application owner",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "apply_stage",
        1
    ),
    boundary!(
        "checked lowered execution-form call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "build_checked_stage_execution_form",
        1
    ),
    boundary!(
        "checked lowered execution-form owner",
        "logic/planner/apply/stage/lowering.rs",
        "../../../../../logic/planner/apply/stage/lowering.rs",
        "build_checked_stage_execution_form",
        1
    ),
    boundary!(
        "legacy lowered execution-form call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "build_legacy_stage_execution_form",
        1
    ),
    boundary!(
        "legacy lowered execution-form owner",
        "logic/planner/apply/stage/lowering.rs",
        "../../../../../logic/planner/apply/stage/lowering.rs",
        "build_legacy_stage_execution_form",
        1
    ),
    boundary!(
        "lowered apply pass owner and call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "run_lowered_apply_pass",
        2
    ),
    boundary!(
        "snapshot publication owner and call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "publish_pending_snapshots",
        2
    ),
    boundary!(
        "stage finalization owner and call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "finalize_stage_results",
        2
    ),
    boundary!(
        "serial semantic finalization call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "finalize_serial_stage_batch",
        2
    ),
    boundary!(
        "serial semantic finalization owner",
        "logic/planner/semantic/finalization.rs",
        "../../../../../logic/planner/semantic/finalization.rs",
        "finalize_serial_stage_batch",
        1
    ),
    boundary!(
        "parallel semantic finalization call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "finalize_stage_batch",
        1
    ),
    boundary!(
        "parallel semantic finalization owner",
        "logic/planner/semantic/finalization.rs",
        "../../../../../logic/planner/semantic/finalization.rs",
        "finalize_stage_batch",
        1
    ),
    boundary!(
        "prepared custom-precompute execution entry",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_prepared_plan_with_precompute",
        1
    ),
    boundary!(
        "scoped scratch-session execution entry",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_evaluation_session_in_scope",
        1
    ),
    boundary!(
        "prepared plan stage-slice executor",
        "logic/planner/execution/mod.rs",
        "../../../../../logic/planner/execution/mod.rs",
        "execute_plan_stage_slices_with_policy",
        3
    ),
    boundary!(
        "stage preparation call",
        "logic/planner/precompute/stage.rs",
        "../../../../../logic/planner/precompute/stage.rs",
        "prepare_epoch",
        1
    ),
    boundary!(
        "checked and legacy epoch preparation owner",
        "logic/planner/precompute/read_preparation.rs",
        "../../../../../logic/planner/precompute/read_preparation.rs",
        "prepare_epoch",
        1
    ),
    boundary!(
        "legacy serial epoch owner",
        "logic/planner/precompute/read_preparation.rs",
        "../../../../../logic/planner/precompute/read_preparation.rs",
        "prepare_legacy_epoch",
        2
    ),
    boundary!(
        "bounded graph epoch admission",
        "logic/planner/precompute/graph_batch.rs",
        "../../../../../logic/planner/precompute/graph_batch.rs",
        "epoch_width",
        1
    ),
    boundary!(
        "checked map lowering call and import",
        "logic/planner/precompute/read_preparation.rs",
        "../../../../../logic/planner/precompute/read_preparation.rs",
        "lower_checked_map",
        2
    ),
    boundary!(
        "checked map lowering owner",
        "logic/planner/precompute/read_preparation/map_declaration.rs",
        "../../../../../logic/planner/precompute/read_preparation/map_declaration.rs",
        "lower_checked_map",
        1
    ),
    boundary!(
        "checked map publication reconciliation",
        "logic/planner/precompute/read_preparation.rs",
        "../../../../../logic/planner/precompute/read_preparation.rs",
        "reconcile_prepared_values",
        2
    ),
    boundary!(
        "graph batch live binding admission",
        "logic/planner/precompute/graph_batch.rs",
        "../../../../../logic/planner/precompute/graph_batch.rs",
        "current_binding",
        3
    ),
    boundary!(
        "grouped concurrent apply call",
        "logic/planner/apply/stage.rs",
        "../../../../../logic/planner/apply/stage.rs",
        "run_grouped_concurrent_apply_pass",
        1
    ),
    boundary!(
        "grouped concurrent apply owner",
        "logic/planner/apply/stage/concurrent.rs",
        "../../../../../logic/planner/apply/stage/concurrent.rs",
        "run_grouped_concurrent_apply_pass",
        1
    ),
    boundary!(
        "parallel apply input lowering",
        "logic/planner/apply/stage/concurrent.rs",
        "../../../../../logic/planner/apply/stage/concurrent.rs",
        "build_concurrent_apply_group_inputs",
        1
    ),
    boundary!(
        "parallel group packet construction",
        "logic/planner/apply/stage/concurrent.rs",
        "../../../../../logic/planner/apply/stage/concurrent.rs",
        "build_group_packet",
        1
    ),
    boundary!(
        "parallel packet reduction",
        "logic/planner/apply/stage/concurrent.rs",
        "../../../../../logic/planner/apply/stage/concurrent.rs",
        "reduce_grouped_concurrent_packets",
        1
    ),
    boundary!(
        "parallel epoch preparation call",
        "logic/planner/apply/stage/concurrent_packets.rs",
        "../../../../../logic/planner/apply/stage/concurrent_packets.rs",
        "prepare_parallel_apply_epoch",
        1
    ),
    boundary!(
        "obsolete group-local publication removed",
        "logic/planner/apply/stage/concurrent_packets.rs",
        "../../../../../logic/planner/apply/stage/concurrent_packets.rs",
        "publish_group_local_task_commit",
        0
    ),
    boundary!(
        "parallel epoch preparation owner",
        "data/graph/runtime/effect/output_commit/epoch.rs",
        "../../../../../data/graph/runtime/effect/output_commit/epoch.rs",
        "prepare_parallel_apply_epoch",
        1
    ),
];
