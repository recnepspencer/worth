pub(crate) use std::collections::{BTreeMap, BTreeSet};

pub(crate) use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
pub(crate) use super::world::supply_chain::{
    commit_branch_batch, lower_supply_chain_production_delta, observe_supply_chain_snapshot,
    snapshot_for_supply_chain_identity, BookingStatus, BranchLabel, DeltaId, EntityKey, EntityKind,
    EntityRecord, SupplyChainProductionDeltaLoweringError, SupplyChainScale,
};
pub(crate) use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::transactions::{
    planned_single_field_locator, AspectFieldPatch, EntityMutationIntent, MutationIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

pub(crate) struct NumberPrestateCase {
    pub(crate) branch: &'static str,
    pub(crate) entity: EntityKey,
    pub(crate) field: &'static str,
    pub(crate) prestate: u64,
    pub(crate) delta: DeltaId,
    pub(crate) observe: fn(&EntityRecord) -> u64,
    pub(crate) expected: u64,
}

pub(crate) fn prove_number_prestate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    case: NumberPrestateCase,
) {
    let NumberPrestateCase {
        branch,
        entity,
        field,
        prestate,
        delta,
        observe,
        expected,
    } = case;
    let (mut world, baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    assert_oracle_matches(execution, &world, &baseline);
    fork(execution, &world.runtime, branch);
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId(branch.to_owned()),
        update_number(&world.handles, entity, field, prestate),
    );
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        &BranchId(branch.to_owned()),
        &BTreeSet::new(),
        delta,
    )
    .expect("actual branch pre-state admits its semantic delta");
    commit_branch_batch(
        execution,
        &world.runtime,
        BranchId(branch.to_owned()),
        batch,
    );
    assert_eq!(
        observe(&observe_branch(execution, &mut world, branch).entities[&entity]),
        expected
    );
}

pub(crate) fn fork(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    branch: &str,
) {
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(branch.to_owned()), source)
        .unwrap();
}

pub(crate) fn update_number(
    handles: &super::world::supply_chain::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: u64,
) -> WorkerIntentBatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new(field).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
        AspectValue::UInt64(value),
    );
    WorkerIntentBatch::new("phase5-actual-prestate-control").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: AspectFieldPatch::new(fields),
        }),
    ))
}

pub(crate) fn update_text(
    handles: &super::world::supply_chain::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: &str,
) -> WorkerIntentBatch {
    let mut fields = BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            AspectKey::new(field).unwrap(),
            FieldKey::new(field).unwrap(),
        ),
        AspectValue::String(worth_foundational::facade::InternedString::Raw(
            value.to_owned(),
        )),
    );
    WorkerIntentBatch::new("phase5-actual-text-prestate-control").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: AspectFieldPatch::new(fields),
        }),
    ))
}

pub(crate) fn observe_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut super::world::supply_chain::ProductionSeededSupplyChainWorld,
    branch: &str,
) -> super::world::supply_chain::ObservedSupplyChainState {
    let identity = world
        .runtime
        .branch_identity(&BranchId(branch.to_owned()))
        .unwrap();
    let snapshot = snapshot_for_supply_chain_identity(&world.runtime, &identity);
    observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(snapshot.clone()),
        &world.runtime,
        &snapshot,
    )
    .unwrap()
}
