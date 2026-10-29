pub(crate) use super::invariant_oracle_expectations::expected_supply_chain_branch;
pub(crate) use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch_with_result, commit_supply_chain_delta,
    compare, fork_supply_chain_branch_from_main, lower_cargo_footprint_batch,
    lower_hazard_v2_batch, observe_supply_chain_snapshot, DeltaId, SupplyChainScale,
};
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
pub(crate) use worth_relational::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
pub(crate) use worth_relational::facade::schema::SchemaVersionId;
pub(crate) use worth_relational::facade::transactions::{RecordRef, WorkerIntentBatch};

pub(crate) fn execute_exact_query(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &super::world::supply_chain::ProductionSeededSupplyChainWorld,
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
