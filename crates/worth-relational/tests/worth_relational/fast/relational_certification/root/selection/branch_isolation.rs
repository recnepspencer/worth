pub(crate) use std::collections::BTreeSet;

pub(crate) use super::invariant_oracle_expectations::expected_supply_chain_branch;
pub(crate) use super::world::supply_chain::{
    assert_oracle_matches, certified_supply_chain_world, commit_branch_batch, compare,
    lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BranchLabel, DeltaId, ObservedSupplyChainState,
    ProductionSeededSupplyChainWorld, SupplyChainScale,
};
pub(crate) use worth_foundational::facade::{AspectKey, AspectValue, FieldKey, InternedString};
pub(crate) use worth_relational::facade::branch::RelationalBranchIdentity;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::inspection::{
    RelationalBranchSharingCostCounters, RelationalMvccCostScope,
};
pub(crate) use worth_relational::facade::query::PlannedQueryPacket;
pub(crate) use worth_relational::facade::runtime::RelationalRuntime;
pub(crate) use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    RecordRef, UpdateEntityFieldsIntent, WorkerIntentBatch,
};

pub(crate) fn assert_child_explicit_query_uses_snapshot_root(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &ProductionSeededSupplyChainWorld,
    child_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    advanced_main_snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
) {
    let expected = world
        .runtime
        .read_truth()
        .read_snapshot(child_snapshot, execution)
        .expect("fixture read request remains live")
        .and_then(|view| {
            view.entities()
                .iter()
                .find(|record| record.entity_id == world.handles.aurora_voyage().id)
                .cloned()
        })
        .expect("child direct read exposes Aurora from its retained root");
    let context = world
        .runtime
        .read_truth()
        .query_plan_context(child_snapshot)
        .expect("child query context remains available");
    let packet = PlannedQueryPacket::explicit_targets(
        "main-advances-before-child-explicit-query",
        context,
        vec![RecordRef::Entity(world.handles.aurora_voyage().id)],
    );
    let plan = world
        .runtime
        .read_truth()
        .plan_query_packet(child_snapshot, packet)
        .expect("child explicit query remains plan-admissible");
    let outcome = world
        .runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("child explicit query executes against its retained root");
    assert_eq!(outcome.result.entities, vec![expected]);

    let main_context = world
        .runtime
        .read_truth()
        .query_plan_context(advanced_main_snapshot)
        .expect("advanced main query context remains available");
    let main_packet = PlannedQueryPacket::explicit_targets(
        "advanced-main-explicit-query-control",
        main_context,
        vec![RecordRef::Entity(world.handles.aurora_voyage().id)],
    );
    let main_plan = world
        .runtime
        .read_truth()
        .plan_query_packet(advanced_main_snapshot, main_packet)
        .expect("advanced main explicit query remains plan-admissible");
    let main_outcome = world
        .runtime
        .read_truth()
        .execute_query_plan(main_plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("advanced main explicit query executes against R1");
    assert_ne!(
        outcome.result.entities, main_outcome.result.entities,
        "Aurora's main-only Held status must distinguish R1 from the child's retained R0"
    );
}

pub(crate) fn advance_main_aurora_status(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut ProductionSeededSupplyChainWorld,
) {
    let status_locator = planned_single_field_locator(
        AspectKey::new("status").expect("Supply Chain status aspect"),
        FieldKey::new("status").expect("Supply Chain status field"),
    );
    let update = UpdateEntityFieldsIntent {
        entity_id: world.handles.aurora_voyage().id,
        fields: AspectFieldPatch::new(std::collections::BTreeMap::from([(
            status_locator,
            AspectValue::String(InternedString::Raw("Held".to_owned())),
        )])),
    };
    let batch = WorkerIntentBatch::new("main-advances-before-child-read").push(
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)),
    );
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId("main".to_owned()),
        batch,
    );
}

pub(crate) fn fork_from_main(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .expect("sibling fork shares the operating root");
}

pub(crate) fn branch_identity(
    runtime: &RelationalRuntime,
    branch: &str,
) -> RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .expect("branch identity is owner-issued")
}

pub(crate) fn observe_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    identity: &RelationalBranchIdentity,
    label: &str,
) -> ObservedSupplyChainState {
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, identity);
    observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap_or_else(|error| panic!("{label} branch remains observable: {error:?}"))
}
