pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch};
pub(crate) use {worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2 as lower_hazard_v2_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::query::DeterministicQueryPlanKey, worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::QueryAccessContract, worth_relational::facade::query::QueryExecutionShape, worth_relational::facade::query::QueryLocalityClass, worth_relational::facade::query::QueryOrderingContract, worth_relational::facade::query::QueryScope, worth_relational::facade::query::ReductionDiscipline};
pub(crate) use {worth_relational::facade::replay::RelationalReplayRequest, worth_relational::facade::replay::ReplayExecutionMode, worth_relational::facade::replay::ReplayVerificationMode};
pub(crate) use {worth_relational::facade::schema::SchemaVersionId};
pub(crate) use {worth_relational::facade::transactions::RecordRef, worth_relational::facade::transactions::WorkerIntentBatch};

pub(crate) fn execute_exact_query(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::production_world::ProductionSeededSupplyChainWorld,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    packet: PlannedQueryPacket,
) -> worth_relational::facade::query::QueryExecutionOutcome {
    let plan = world
        .runtime
        .read_truth()
        .plan_query_packet(snapshot, packet)
        .expect("the exact V2 query plan is admitted");
    world
        .runtime
        .read_truth()
        .execute_query_plan(plan, execution)
        .expect("request remains live")
        .map(|read| read.outcome)
        .expect("the exact V2 query plan executes")
}

pub(crate) fn assert_schema_v2(outcome: &worth_relational::facade::query::QueryExecutionOutcome) {
    assert!(outcome
        .result
        .entities
        .iter()
        .all(|record| record.kind.schema_version_id == SchemaVersionId(2)));
    assert!(outcome
        .result
        .relations
        .iter()
        .all(|record| record.kind.schema_version_id == SchemaVersionId(2)));
}

pub(crate) fn assert_matches_retained_root(
    outcome: &worth_relational::facade::query::QueryExecutionOutcome,
    entities: &std::collections::BTreeMap<
        worth_relational::facade::identity::EntityId,
        worth_relational::facade::runtime::EntityReadRecord,
    >,
    relations: &std::collections::BTreeMap<
        worth_relational::facade::identity::RelationId,
        worth_relational::facade::runtime::RelationReadRecord,
    >,
) {
    for record in &outcome.result.entities {
        assert_eq!(entities.get(&record.entity_id), Some(record));
    }
    for record in &outcome.result.relations {
        assert_eq!(relations.get(&record.relation_id), Some(record));
    }
}