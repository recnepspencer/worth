pub(crate) use std::collections::BTreeSet;

pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::assert_oracle_matches, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::lowering::lower_supply_chain_production_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::snapshot_for_supply_chain_identity, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison_state::ObservedSupplyChainState, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_foundational::facade::AspectKey, worth_foundational::facade::AspectValue, worth_foundational::facade::FieldKey, worth_foundational::facade::InternedString};
pub(crate) use {worth_relational::facade::branch::RelationalBranchIdentity};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::inspection::RelationalBranchSharingCostCounters, worth_relational::facade::inspection::RelationalMvccCostScope};
pub(crate) use {worth_relational::facade::query::PlannedQueryPacket};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::planned_single_field_locator, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::transactions::EntityMutationIntent, worth_relational::facade::transactions::MutationIntent, worth_relational::facade::transactions::RecordRef, worth_relational::facade::transactions::UpdateEntityFieldsIntent, worth_relational::facade::transactions::WorkerIntentBatch};

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
    let packet = worth_relational::facade::query::PlannedQueryPacket::explicit_targets(
        "main-advances-before-child-explicit-query",
        context,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            world.handles.aurora_voyage().id,
        )],
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
    let main_packet = worth_relational::facade::query::PlannedQueryPacket::explicit_targets(
        "advanced-main-explicit-query-control",
        main_context,
        vec![worth_relational::facade::transactions::RecordRef::Entity(
            world.handles.aurora_voyage().id,
        )],
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
        worth_foundational::facade::AspectKey::new("status").expect("Supply Chain status aspect"),
        worth_foundational::facade::FieldKey::new("status").expect("Supply Chain status field"),
    );
    let update = UpdateEntityFieldsIntent {
        entity_id: world.handles.aurora_voyage().id,
        fields: worth_relational::facade::transactions::AspectFieldPatch::new(
            std::collections::BTreeMap::from([(
                status_locator,
                worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw("Held".to_owned()),
                ),
            )]),
        ),
    };
    let batch = worth_relational::facade::transactions::WorkerIntentBatch::new(
        "main-advances-before-child-read",
    )
    .push(
        worth_relational::facade::transactions::MutationIntent::Entity(
            worth_relational::facade::transactions::EntityMutationIntent::UpdateFields(update),
        ),
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
    world: &mut worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
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
