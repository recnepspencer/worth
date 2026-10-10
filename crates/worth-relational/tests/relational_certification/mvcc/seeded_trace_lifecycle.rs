use super::{trace_panic, BranchModelState, ProductionModelTrace};
use crate::world::supply_chain::{
    observe_supply_chain_snapshot, ObservedSupplyChainState, OracleBranch, OracleState,
    ProductionSeededSupplyChainWorld,
};
use worth_relational::facade::branch::{
    RelationalBranchLifecyclePosture, RelationalBranchReferenceState,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::publication::PatchStreamRequest;
use worth_relational::facade::snapshots::SnapshotHandle;

pub(super) fn archive_and_observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    state: &BranchModelState,
) -> SnapshotHandle {
    let identity = world
        .runtime
        .branch_identity(&state.branch)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "archive identity", error));
    let (_, basis) = world
        .runtime
        .observe_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "archive basis", error));
    let retained = world
        .runtime
        .retain_component_basis(&basis)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "retain basis", error));
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "archive snapshot", error));
    world
        .runtime
        .archive_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "archive", error));
    let reference = world
        .runtime
        .branch_reference_state(&state.branch)
        .unwrap_or_else(|| trace_panic(trace, scenario_index, "archive reference", &state.branch));
    if reference.lifecycle_posture() != RelationalBranchLifecyclePosture::Archived {
        trace_panic(trace, scenario_index, "archive posture", reference);
    }
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        &state.oracle,
        "archive oracle",
    );
    world
        .runtime
        .release_component_basis(retained)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "release basis", error));
    let retention = world
        .runtime
        .branch_retention_cost_counters(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "archive retention", error));
    if retention.external_pin_acquires != retention.external_pin_releases {
        trace_panic(trace, scenario_index, "external lease release", retention);
    }
    snapshot
}

pub(super) fn delete_and_prove_absence(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    branches: &[BranchModelState],
) {
    let state = &branches[scenario_index];
    let identity = world
        .runtime
        .branch_identity(&state.branch)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "delete identity", error));
    let (_, basis) = world
        .runtime
        .observe_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "delete basis", error));
    let root = basis.observation().selected_root_identity();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "delete snapshot", error));
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        &state.oracle,
        "pre-delete oracle",
    );
    let before = observe_snapshot(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        "pre-delete truth",
    );
    let catalog = world.runtime.history().immutable_commit_count();
    let replay = world.runtime.publication().latest_replay();
    let stream = world
        .runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "delete patch stream", error));
    let survivors = survivor_references(world, branches, &state.branch);
    let deleted = world
        .runtime
        .delete_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "delete", error))
        .deleted()
        .cloned()
        .unwrap_or_else(|| trace_panic(trace, scenario_index, "delete settlement", &state.branch));
    let stream_after = world
        .runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .unwrap_or_else(|error| {
            trace_panic(trace, scenario_index, "delete patch stream after", error)
        });
    if deleted.retired_root_identity() != root
        || world.runtime.branch_identity(&state.branch).is_ok()
        || world
            .runtime
            .branch_reference_state(&state.branch)
            .is_some()
        || world
            .runtime
            .branch_retention_cost_counters(&identity)
            .is_ok()
        || world.runtime.history().immutable_commit_count() != catalog
        || world.runtime.publication().latest_replay() != replay
        || stream_after != stream
        || survivor_references(world, branches, &state.branch) != survivors
    {
        trace_panic(trace, scenario_index, "delete residue", &deleted);
    }
    prove_surviving_truth(
        execution,
        world,
        trace,
        scenario_index,
        branches,
        &state.branch,
    );
    let inspection = world
        .runtime
        .read_truth()
        .inspect_snapshot(&snapshot)
        .unwrap_or_else(|| trace_panic(trace, scenario_index, "deleted root inspection", root));
    if inspection.root_id != Some(root) {
        trace_panic(trace, scenario_index, "deleted root lease", inspection);
    }
    let after = observe_snapshot(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        "deleted root truth",
    );
    compare_retained_truth(trace, scenario_index, &before, &after);
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "deleted lease release", error));
}

fn prove_surviving_truth(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    branches: &[BranchModelState],
    excluded: &BranchId,
) {
    let main = OracleBranch::genesis(OracleState::from_definition(world.program.definition()));
    observe_live_branch(
        execution,
        world,
        trace,
        scenario_index,
        &BranchId("main".to_owned()),
        &main,
    );
    for state in branches.iter().filter(|state| &state.branch != excluded) {
        if let Some(snapshot) = &state.retained_snapshot {
            super::seeded_trace_observation::observe_snapshot_against_oracle(
                execution,
                world,
                trace,
                scenario_index,
                snapshot,
                &state.oracle,
                "archived survivor oracle",
            );
        } else {
            observe_live_branch(
                execution,
                world,
                trace,
                scenario_index,
                &state.branch,
                &state.oracle,
            );
        }
    }
}

fn observe_live_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    branch: &BranchId,
    oracle: &OracleBranch,
) {
    let identity = world
        .runtime
        .branch_identity(branch)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "survivor identity", error));
    let (_, basis) = world
        .runtime
        .observe_branch(&identity)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "survivor basis", error));
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "survivor snapshot", error));
    super::seeded_trace_observation::observe_snapshot_against_oracle(
        execution,
        world,
        trace,
        scenario_index,
        &snapshot,
        oracle,
        "survivor oracle",
    );
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap_or_else(|error| trace_panic(trace, scenario_index, "survivor release", error));
}

fn survivor_references(
    world: &ProductionSeededSupplyChainWorld,
    branches: &[BranchModelState],
    excluded: &BranchId,
) -> Vec<(BranchId, RelationalBranchReferenceState)> {
    std::iter::once(BranchId("main".to_owned()))
        .chain(
            branches
                .iter()
                .filter(|state| &state.branch != excluded)
                .map(|state| state.branch.clone()),
        )
        .filter_map(|branch| {
            world
                .runtime
                .branch_reference_state(&branch)
                .map(|state| (branch, state))
        })
        .collect()
}

fn observe_snapshot(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    trace: &ProductionModelTrace,
    scenario_index: usize,
    snapshot: &SnapshotHandle,
    operation: &'static str,
) -> ObservedSupplyChainState {
    observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        snapshot,
    )
    .unwrap_or_else(|error| trace_panic(trace, scenario_index, operation, error))
}

fn compare_retained_truth(
    trace: &ProductionModelTrace,
    scenario_index: usize,
    before: &ObservedSupplyChainState,
    after: &ObservedSupplyChainState,
) {
    if before.schema != after.schema
        || before.entities != after.entities
        || before.relations != after.relations
        || before.relation_vector != after.relation_vector
        || before.absent_entities != after.absent_entities
        || before.absent_relations != after.absent_relations
    {
        trace_panic(
            trace,
            scenario_index,
            "deleted immutable truth",
            (before, after),
        );
    }
}
