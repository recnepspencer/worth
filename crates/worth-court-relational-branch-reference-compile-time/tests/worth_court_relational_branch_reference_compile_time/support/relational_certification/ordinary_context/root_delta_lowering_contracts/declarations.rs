pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::BookingStatus, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::EntityRecord, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::SupplyChainProductionDeltaLoweringError, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

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
    let (mut world, baseline) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
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
        &std::collections::BTreeSet::new(),
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
    handles: &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: u64,
) -> WorkerIntentBatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new(field).unwrap(),
            worth_foundational::facade::FieldKey::new(field).unwrap(),
        ),
        worth_foundational::facade::AspectValue::UInt64(value),
    );
    worth_relational::facade::transactions::WorkerIntentBatch::new("phase5-actual-prestate-control").push(worth_relational::facade::transactions::MutationIntent::Entity(
        worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: worth_relational::facade::transactions::AspectFieldPatch::new(fields),
        }),
    ))
}

pub(crate) fn update_text(
    handles: &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::handles::SupplyChainSemanticHandles,
    entity: EntityKey,
    field: &str,
    value: &str,
) -> WorkerIntentBatch {
    let mut fields = std::collections::BTreeMap::new();
    fields.insert(
        planned_single_field_locator(
            worth_foundational::facade::AspectKey::new(field).unwrap(),
            worth_foundational::facade::FieldKey::new(field).unwrap(),
        ),
        worth_foundational::facade::AspectValue::String(worth_foundational::facade::InternedString::Raw(
            value.to_owned(),
        )),
    );
    worth_relational::facade::transactions::WorkerIntentBatch::new("phase5-actual-text-prestate-control").push(worth_relational::facade::transactions::MutationIntent::Entity(
        worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
            entity_id: handles.entities[&entity].id,
            fields: worth_relational::facade::transactions::AspectFieldPatch::new(fields),
        }),
    ))
}

pub(crate) fn observe_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
    branch: &str,
) -> worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState {
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