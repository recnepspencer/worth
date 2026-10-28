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

#[test]
fn v2_latest_replay_matches_the_canonical_envelope_and_reconstructs() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let branch = BranchId("main".to_owned());
    let batch = lower_hazard_v2_batch(&world.handles).unwrap();
    let committed = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        DeltaId::AdoptHazardClassificationV2,
        batch,
    );
    let envelope = world
        .runtime
        .replay()
        .canonical_commit_envelope(execution, committed.commit.commit_id)
        .unwrap()
        .unwrap()
        .clone();
    let latest = world.runtime.publication().latest_replay().unwrap().clone();
    assert_eq!(latest.commit_id, committed.commit.commit_id);
    assert_eq!(latest.schema_authority, envelope.schema_authority);
    let replay = world
        .runtime
        .replay_authority()
        .replay_commit(
            RelationalReplayRequest {
                commit_id: committed.commit.commit_id,
                branch_id: branch,
                execution_mode: ReplayExecutionMode::SerialDeterministic,
                verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
            },
            execution,
        )
        .unwrap();
    assert_eq!(
        replay.failure, None,
        "V2 replay must reconstruct through envelope authority: {replay:?}"
    );
    assert!(replay.mismatches.is_empty());
    world
        .runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}

#[test]
fn retained_v2_snapshot_materializes_kind_metadata_from_its_root_authority() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let branch = BranchId("main".to_owned());
    let transitioned = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        DeltaId::AdoptHazardClassificationV2,
        lower_hazard_v2_batch(&world.handles).unwrap(),
    );
    let successor = commit_branch_batch_with_result(
        execution,
        &world.runtime,
        branch,
        WorkerIntentBatch::new("supply-chain-v2-successor"),
    );

    let read = world
        .runtime
        .read_truth()
        .read_snapshot(&transitioned.snapshot, execution)
        .expect("fixture read request remains live")
        .expect("the retained V2 snapshot remains readable after its branch advances");
    assert!(!read.entities().is_empty());
    assert!(!read.relations().is_empty());
    assert!(read
        .entities()
        .iter()
        .all(|record| record.kind.schema_version_id == SchemaVersionId(2)));
    assert!(read
        .relations()
        .iter()
        .all(|record| record.kind.schema_version_id == SchemaVersionId(2)));
    assert_eq!(
        world
            .runtime
            .read_truth()
            .snapshot_schema_version(&transitioned.snapshot),
        Some(SchemaVersionId(2))
    );

    world
        .runtime
        .snapshots()
        .release_snapshot(&transitioned.snapshot)
        .unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&successor.snapshot)
        .unwrap();
}

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
