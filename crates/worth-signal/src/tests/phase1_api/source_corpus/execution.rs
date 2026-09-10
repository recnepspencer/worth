pub(in crate::tests::phase1_api) const HOT_APPLY_SOURCE: &str = concat!(
    include_str!("../../../logic/evaluation/engine/apply.rs"),
    include_str!("../../../logic/evaluation/engine/apply/dependency_inputs.rs"),
    include_str!("../../../logic/evaluation/engine/apply/effect_lowering.rs"),
    include_str!("../../../logic/evaluation/engine/apply/mutation.rs"),
    include_str!("../../../logic/evaluation/engine/apply/telemetry.rs"),
    include_str!("../../../logic/evaluation/engine/apply/verdict.rs"),
);
pub(in crate::tests::phase1_api) const HOT_PREPARED_APPLY_SOURCE: &str = concat!(
    include_str!("../../../logic/evaluation/engine/prepared_apply.rs"),
    include_str!("../../../logic/evaluation/engine/prepared_apply/admission.rs"),
    include_str!("../../../logic/evaluation/engine/prepared_apply/evaluation.rs"),
    include_str!("../../../logic/evaluation/engine/prepared_apply/input.rs"),
    include_str!("../../../logic/evaluation/engine/prepared_apply/parallel.rs"),
    include_str!("../../../logic/evaluation/engine/prepared_apply/telemetry.rs"),
);
pub(in crate::tests::phase1_api) const HOT_SEMANTIC_FINALIZE_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/semantic/mod.rs"),
    include_str!("../../../logic/planner/semantic/artifacts.rs"),
    include_str!("../../../logic/planner/semantic/finalization.rs"),
    include_str!("../../../logic/planner/semantic/reporting.rs"),
    include_str!("../../../logic/planner/semantic/segments.rs"),
    include_str!("../../../logic/planner/semantic/stage_recording.rs"),
);
pub(in crate::tests::phase1_api) const HOT_EFFECT_SOURCE: &str = concat!(
    include_str!("../../../data/graph/runtime/effect.rs"),
    include_str!("../../../data/graph/runtime/effect/admission.rs"),
    include_str!("../../../data/graph/runtime/effect/output_commit/snapshot_preparation.rs"),
    include_str!("../../../data/graph/runtime/effect/batching.rs"),
    include_str!("../../../data/graph/runtime/effect/evidence.rs"),
    include_str!("../../../data/graph/runtime/effect/vocabulary.rs"),
);
pub(in crate::tests::phase1_api) const HOT_SERIAL_BATCH_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/apply/serial_batch.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/application.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/finalization.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/lowered_stage.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/preparation.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/task_lowering.rs"),
    include_str!("../../../logic/planner/apply/serial_batch/witness.rs"),
);
pub(in crate::tests::phase1_api) const HOT_STAGE_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/apply/stage.rs"),
    include_str!("../../../logic/planner/apply/stage/concurrent.rs"),
    include_str!("../../../logic/planner/apply/stage/concurrent_packets.rs"),
    include_str!("../../../logic/planner/apply/stage/footprint.rs"),
    include_str!("../../../logic/planner/apply/stage/lowering.rs"),
    include_str!("../../../logic/planner/apply/stage/metrics.rs"),
    include_str!("../../../logic/planner/apply/stage/strategy.rs"),
);
pub(in crate::tests::phase1_api) const HOT_PLANNING_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/planning/mod.rs"),
    include_str!("../../../logic/planner/planning/admission.rs"),
    include_str!("../../../logic/planner/planning/evidence.rs"),
    include_str!("../../../logic/planner/planning/stage_formation.rs"),
    include_str!("../../../logic/planner/planning/topology.rs"),
    include_str!("../../../logic/planner/planning/validation.rs"),
);
pub(in crate::tests::phase1_api) const HOT_VALIDATION_SOURCE: &str =
    include_str!("../../../logic/planner/planning/validation.rs");
pub(in crate::tests::phase1_api) const HOT_PRECOMPUTE_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/precompute/mod.rs"),
    include_str!("../../../logic/planner/precompute/admission.rs"),
    include_str!("../../../logic/planner/precompute/dispatch.rs"),
    include_str!("../../../logic/planner/precompute/eligibility.rs"),
    include_str!("../../../logic/planner/precompute/executor_pool.rs"),
    include_str!("../../../logic/planner/precompute/read_preparation.rs"),
    include_str!("../../../logic/planner/precompute/reporting.rs"),
    include_str!("../../../logic/planner/precompute/stage.rs"),
    include_str!("../../../logic/planner/precompute/stage_data.rs"),
    include_str!("../../../logic/planner/precompute/temporal.rs"),
);
pub(in crate::tests::phase1_api) const HOT_CONTEXT_SOURCE: &str =
    include_str!("../../../logic/context.rs");
pub(in crate::tests::phase1_api) const HOT_REUSE_CONTEXT_SOURCE: &str =
    include_str!("../../../logic/evaluation/reuse/context_resolution.rs");
pub(in crate::tests::phase1_api) const HOT_INVALIDATION_ROUTING_SOURCE: &str = concat!(
    include_str!("../../../logic/invalidation/routing.rs"),
    include_str!("../../../logic/invalidation/routing/application.rs"),
    include_str!("../../../logic/invalidation/routing/counters.rs"),
    include_str!("../../../logic/invalidation/routing/evidence.rs"),
    include_str!("../../../logic/invalidation/routing/planning.rs"),
    include_str!("../../../logic/invalidation/routing/seeds.rs"),
);
pub(in crate::tests::phase1_api) const HOT_INVALIDATION_SUBSCRIPTION_SOURCE: &str =
    include_str!("../../../logic/invalidation/subscription.rs");
pub(in crate::tests::phase1_api) const HOT_TRANSACTION_OBSERVATION_MUTATION_SOURCE: &str =
    include_str!("../../../logic/transaction/runtime/transaction/transaction_mutation.rs");
pub(in crate::tests::phase1_api) const HOT_EASY_OBSERVATION_SOURCE: &str =
    include_str!("../../../easy/observation.rs");
pub(in crate::tests::phase1_api) const HOT_RUNTIME_OBSERVATION_SOURCE: &str = concat!(
    include_str!("../../../logic/transaction/runtime/state/runtime_observation.rs"),
    include_str!("../../../logic/transaction/runtime/state/runtime_observation/registry.rs"),
);
pub(in crate::tests::phase1_api) const PROOF_SOURCE: &str = concat!(
    include_str!("../../../data/proof/mod.rs"),
    include_str!("../../../data/proof/snapshot_commit.rs"),
);
pub(in crate::tests::phase1_api) const PLANNER_MODEL_SOURCE: &str = concat!(
    include_str!("../../../logic/planner/model/mod.rs"),
    include_str!("../../../logic/planner/model/admission.rs"),
    include_str!("../../../logic/planner/model/apply.rs"),
    include_str!("../../../logic/planner/model/frontier_route_receipt.rs"),
    include_str!("../../../logic/planner/model/plan.rs"),
    include_str!("../../../logic/planner/model/report.rs"),
    include_str!("../../../logic/planner/model/strategy.rs"),
    include_str!("../../../logic/planner/model/task.rs"),
);
pub(in crate::tests::phase1_api) const SEMANTIC_SOURCE: &str = HOT_SEMANTIC_FINALIZE_SOURCE;
pub(in crate::tests::phase1_api) const WORKSPACE_SOURCE: &str =
    include_str!("../../../logic/planner/apply/workspace.rs");
pub(in crate::tests::phase1_api) const PATCH_BUFFER_SOURCE: &str =
    include_str!("../../../logic/transaction/patch_buffer.rs");
pub(in crate::tests::phase1_api) const ORDINARY_INVALIDATION_ACCESS_SOURCE: &str = concat!(
    include_str!("../../../logic/invalidation/causality/dependency_admission.rs"),
    include_str!("../../../logic/invalidation/scheduling/readiness.rs"),
);
pub(in crate::tests::phase1_api) const INVALIDATION_REVALIDATION_SOURCE: &str =
    include_str!("../../../logic/invalidation/causality/revalidation.rs");
pub(in crate::tests::phase1_api) const ORDINARY_EXPLANATION_ACCESS_SOURCE: &str = concat!(
    include_str!("../../../logic/explain/analysis.rs"),
    include_str!("../../../logic/explain/resolver/assembly.rs"),
    include_str!("../../../logic/explain/resolver/causes.rs"),
    include_str!("../../../logic/explain/resolver/policy.rs"),
);
