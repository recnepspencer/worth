#[derive(Clone, Copy)]
struct ExactBoundarySymbol {
    responsibility: &'static str,
    source_path: &'static str,
    source: &'static str,
    symbol: &'static str,
    exact_occurrences: usize,
}

macro_rules! boundary {
    ($responsibility:literal, $path:literal, $source:literal, $symbol:literal, $count:literal) => {
        ExactBoundarySymbol {
            responsibility: $responsibility,
            source_path: $path,
            source: include_str!($source),
            symbol: $symbol,
            exact_occurrences: $count,
        }
    };
}

mod constructors;
mod counters;
mod owners;
mod planner;
use counters::COUNTER_AND_EXPORT_BOUNDARIES;
use planner::PLANNER_AND_EXECUTOR_BOUNDARIES;

const ADAPTER_EXPORTS: &str = include_str!("../../../../facade/adapters.rs");
const INTEGRATION_EXPORTS: &str = include_str!("../../../../facade/integration.rs");

const OPERATIONAL_BOUNDARIES: &[ExactBoundarySymbol] = &[
    boundary!(
        "source seed preparation",
        "logic/invalidation/routing/seeds.rs",
        "../../../../logic/invalidation/routing/seeds.rs",
        "prepare_invalidation_seed_batch",
        1
    ),
    boundary!(
        "producer-wide subscriber scan removed from source routing",
        "logic/invalidation/routing/seeds.rs",
        "../../../../logic/invalidation/routing/seeds.rs",
        "runtime_subscribers_of",
        0
    ),
    boundary!(
        "source-only frontier planning",
        "logic/invalidation/routing/planning.rs",
        "../../../../logic/invalidation/routing/planning.rs",
        "plan_invalidation_frontier",
        1
    ),
    boundary!(
        "contract admission removed from source routing",
        "logic/invalidation/routing/planning.rs",
        "../../../../logic/invalidation/routing/planning.rs",
        "cares_about_change",
        0
    ),
    boundary!(
        "direct grouping removed from source routing",
        "logic/invalidation/routing/planning.rs",
        "../../../../logic/invalidation/routing/planning.rs",
        "collect_direct_groups",
        0
    ),
    boundary!(
        "transitive closure removed from source routing",
        "logic/invalidation/routing/application.rs",
        "../../../../logic/invalidation/routing/application.rs",
        "execute_transitive_wave",
        0
    ),
    boundary!(
        "source-only frontier execution",
        "logic/invalidation/routing/application.rs",
        "../../../../logic/invalidation/routing/application.rs",
        "execute_invalidation_frontier",
        1
    ),
    boundary!(
        "dirty batch orchestration entry",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "mark_dirty_batch",
        3
    ),
    boundary!(
        "frontier planning orchestration",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "plan_invalidation_frontier",
        3
    ),
    boundary!(
        "frontier application orchestration",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "execute_invalidation_frontier",
        3
    ),
    boundary!(
        "retained trace orchestration",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "retained_trace_records",
        3
    ),
    boundary!(
        "legacy direct entry mutation seam removed",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "apply_direct_entry",
        0
    ),
    boundary!(
        "source seed mutation seam",
        "logic/invalidation/routing.rs",
        "../../../../logic/invalidation/routing.rs",
        "mark_source_seed",
        1
    ),
    boundary!(
        "prepared output commit seam",
        "data/graph/runtime/effect/output_commit.rs",
        "../../../../data/graph/runtime/effect/output_commit.rs",
        "prepare_output_commit_packet",
        3
    ),
    boundary!(
        "performed output publication seam",
        "data/graph/runtime/effect/output_commit.rs",
        "../../../../data/graph/runtime/effect/output_commit.rs",
        "publish_output_commit_packet",
        3
    ),
    boundary!(
        "checkpoint cause readmission",
        "logic/invalidation/causality/revalidation.rs",
        "../../../../logic/invalidation/causality/revalidation.rs",
        "readmit_checkpoint_causes",
        1
    ),
    boundary!(
        "operational invalidation input",
        "logic/invalidation/causality/revalidation.rs",
        "../../../../logic/invalidation/causality/revalidation.rs",
        "node_invalidation_input",
        1
    ),
    boundary!(
        "direct cause packet validation",
        "logic/invalidation/causality/dependency_admission.rs",
        "../../../../logic/invalidation/causality/dependency_admission.rs",
        "validate_packet",
        1
    ),
    boundary!(
        "direct output cause preparation",
        "logic/invalidation/causality/dependency_admission.rs",
        "../../../../logic/invalidation/causality/dependency_admission.rs",
        "prepare_direct_output_causes",
        1
    ),
    boundary!(
        "stable output settlement preparation",
        "logic/invalidation/causality/dependency_admission.rs",
        "../../../../logic/invalidation/causality/dependency_admission.rs",
        "prepare_stable_output_resolution",
        1
    ),
    boundary!(
        "direct output cause publication",
        "logic/invalidation/causality/dependency_admission/publication.rs",
        "../../../../logic/invalidation/causality/dependency_admission/publication.rs",
        "publish_direct_output_causes",
        1
    ),
    boundary!(
        "edge-cause reconciliation calls",
        "logic/invalidation/causality/dependency_admission.rs",
        "../../../../logic/invalidation/causality/dependency_admission.rs",
        "reconcile_edge_cause",
        2
    ),
    boundary!(
        "edge-cause reconciliation owner",
        "logic/invalidation/causality/cause_aggregation.rs",
        "../../../../logic/invalidation/causality/cause_aggregation.rs",
        "reconcile_edge_cause",
        1
    ),
];

const RESTORE_BOUNDARIES: &[ExactBoundarySymbol] = &[
    boundary!(
        "raw checkpoint authority restore",
        "data/graph/runtime/graph/checkpoint.rs",
        "../../../../data/graph/runtime/graph/checkpoint.rs",
        "restore_from_checkpoint_authority",
        2
    ),
    boundary!(
        "supported checkpoint image restore",
        "data/graph/runtime/graph/checkpoint.rs",
        "../../../../data/graph/runtime/graph/checkpoint.rs",
        "restore_from_checkpoint_image",
        1
    ),
    boundary!(
        "checkpoint topology reconstruction",
        "data/graph/runtime/graph/checkpoint.rs",
        "../../../../data/graph/runtime/graph/checkpoint.rs",
        "rebuild_checkpoint_topology",
        2
    ),
    boundary!(
        "snapshot authority graph restore",
        "state/snapshot.rs",
        "../../../../state/snapshot.rs",
        "authority_graph",
        1
    ),
    boundary!(
        "branch snapshot materialization",
        "logic/transaction/runtime/state/branching/fork_snapshot.rs",
        "../../../../logic/transaction/runtime/state/branching/fork_snapshot.rs",
        "materialize_snapshot_fork_state",
        1
    ),
    boundary!(
        "branch snapshot authority readmission",
        "logic/transaction/runtime/state/branching/fork_snapshot.rs",
        "../../../../logic/transaction/runtime/state/branching/fork_snapshot.rs",
        "authority_graph",
        1
    ),
];

const REMOVED_PUBLIC_FRONTIER_TYPES: &[&str] = &[
    "FrontierWaveEntryPlan",
    "FrontierWavePlan",
    "FrontierPredictedCounters",
    "FrontierPlan",
    "FrontierWaveEntrySummary",
    "FrontierWaveSummary",
    "TransitiveFrontierEntrySummary",
    "TransitiveFrontierWaveSummary",
    "FrontierExecutionCounters",
    "FrontierExecutionSummary",
];

const CURRENT_PUBLIC_INVALIDATION_TYPES: &[&str] = &[
    "InvalidationPlanningEstimate",
    "SignalInvalidationExecutionObservation",
    "SignalInvalidationExecutionReceipt",
    "SignalInvalidationRealizedCounters",
    "InvalidationExecutionSummary",
];

fn assert_exact_inventory(boundaries: &[ExactBoundarySymbol]) {
    for boundary in boundaries {
        assert_eq!(
            exact_token_occurrences(boundary.source, boundary.symbol),
            boundary.exact_occurrences,
            "{} ({}) changed its exact symbol footprint",
            boundary.responsibility,
            boundary.source_path
        );
    }
}

fn exact_token_occurrences(source: &str, symbol: &str) -> usize {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| *token == symbol)
        .count()
}

#[test]
fn phase_1_inventory_freezes_current_operational_cutover_boundaries() {
    assert_eq!(OPERATIONAL_BOUNDARIES.len(), 23);
    assert_exact_inventory(OPERATIONAL_BOUNDARIES);
}

#[test]
fn phase_1_inventory_freezes_restore_and_branch_readmission_boundaries() {
    assert_eq!(RESTORE_BOUNDARIES.len(), 6);
    assert_exact_inventory(RESTORE_BOUNDARIES);
}

#[test]
fn phase_1_inventory_freezes_counter_topology_and_export_authorities() {
    assert_eq!(COUNTER_AND_EXPORT_BOUNDARIES.len(), 18);
    assert_exact_inventory(COUNTER_AND_EXPORT_BOUNDARIES);
}

#[test]
fn phase_1_inventory_freezes_planner_readiness_and_executor_seams() {
    assert_eq!(PLANNER_AND_EXECUTOR_BOUNDARIES.len(), 38);
    assert_exact_inventory(PLANNER_AND_EXECUTOR_BOUNDARIES);
}

#[test]
fn phase_6_inventory_proves_public_cutover_and_old_surface_removal() {
    assert_eq!(REMOVED_PUBLIC_FRONTIER_TYPES.len(), 10);
    for removed in REMOVED_PUBLIC_FRONTIER_TYPES {
        assert_eq!(exact_token_occurrences(ADAPTER_EXPORTS, removed), 0,);
        assert_eq!(exact_token_occurrences(INTEGRATION_EXPORTS, removed), 0,);
    }
    for current in CURRENT_PUBLIC_INVALIDATION_TYPES {
        assert_eq!(exact_token_occurrences(ADAPTER_EXPORTS, current), 1);
        assert_eq!(exact_token_occurrences(INTEGRATION_EXPORTS, current), 1);
    }
    assert_eq!(
        CURRENT_PUBLIC_INVALIDATION_TYPES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        CURRENT_PUBLIC_INVALIDATION_TYPES.len()
    );
}
