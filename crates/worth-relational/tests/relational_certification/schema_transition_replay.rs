use super::invariant_oracle_expectations::expected_supply_chain_branch;
use super::world::supply_chain::{
    certified_supply_chain_world, commit_branch_batch_with_result, commit_supply_chain_delta,
    compare, fork_supply_chain_branch_from_main, lower_cargo_footprint_batch,
    lower_hazard_v2_batch, observe_supply_chain_snapshot, DeltaId, SupplyChainScale,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::query::{
    DeterministicQueryPlanKey, PlannedQueryPacket, QueryAccessContract, QueryExecutionShape,
    QueryLocalityClass, QueryOrderingContract, QueryScope, ReductionDiscipline,
};
use worth_relational::facade::replay::{
    RelationalReplayRequest, ReplayExecutionMode, ReplayVerificationMode,
};
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::transactions::{RecordRef, WorkerIntentBatch};

#[test]
fn v2_latest_replay_matches_the_canonical_envelope_and_reconstructs() {
    let host_request = crate::test_execution_policy::serial_request();
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
        .canonical_commit_envelope(committed.commit.commit_id)
        .unwrap()
        .clone();
    let latest = world.runtime.publication().latest_replay().unwrap().clone();
    assert_eq!(latest.commit_id, committed.commit.commit_id);
    assert_eq!(latest.schema_authority, envelope.schema_authority);
    let replay = world.runtime.replay_authority().replay_commit(
        RelationalReplayRequest {
            commit_id: committed.commit.commit_id,
            branch_id: branch,
            execution_mode: ReplayExecutionMode::SerialDeterministic,
            verification_mode: ReplayVerificationMode::NormalRecoveryVerification,
        },
        execution,
    );
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
    let host_request = crate::test_execution_policy::serial_request();
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

#[test]
fn retained_v2_query_plans_materialize_from_their_exact_root_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let branch = BranchId("hazard-v2".to_owned());
    fork_supply_chain_branch_from_main(&world.runtime, branch.clone());
    let transitioned = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        DeltaId::AdoptHazardClassificationV2,
        lower_hazard_v2_batch(&world.handles).unwrap(),
    );
    let retained_observed = observe_supply_chain_snapshot(
        execution,
        &world.program,
        &world.handles.for_snapshot(transitioned.snapshot.clone()),
        &world.runtime,
        &transitioned.snapshot,
    )
    .expect("the retained V2 root is publicly observable");
    compare(
        &expected_supply_chain_branch(
            &world.program,
            super::world::supply_chain::BranchLabel::HazardV2,
            Some(DeltaId::AdoptHazardClassificationV2),
        ),
        &retained_observed,
    )
    .expect("the retained V2 root matches the complete independent oracle");
    let retained_view = world
        .runtime
        .read_truth()
        .read_snapshot(&transitioned.snapshot, execution)
        .expect("fixture read request remains live")
        .expect("the retained V2 snapshot supplies exact query targets");
    let cargo_id = world.handles.medical_cargo().id;
    let entity = retained_view
        .entities()
        .iter()
        .find(|record| record.entity_id == cargo_id)
        .expect("the retained root contains the named cargo")
        .clone();
    let relation = retained_view
        .relations()
        .iter()
        .find(|record| record.source == cargo_id)
        .expect("the retained root contains an outgoing cargo relation")
        .clone();
    let expected_kind_entities = retained_view
        .entities()
        .iter()
        .filter(|record| {
            record.kind.kind_id == entity.kind.kind_id
                && record.entity_id.partition_id == entity.entity_id.partition_id
        })
        .cloned()
        .collect::<Vec<_>>();
    let retained_entities = retained_view
        .entities()
        .iter()
        .cloned()
        .map(|record| (record.entity_id, record))
        .collect::<std::collections::BTreeMap<_, _>>();
    let retained_relations = retained_view
        .relations()
        .iter()
        .cloned()
        .map(|record| (record.relation_id, record))
        .collect::<std::collections::BTreeMap<_, _>>();
    drop(retained_view);
    let successor = commit_branch_batch_with_result(
        execution,
        &world.runtime,
        branch,
        lower_cargo_footprint_batch(&world.handles, SupplyChainScale::court(), 1),
    );
    let successor_view = world
        .runtime
        .read_truth()
        .read_snapshot(&successor.snapshot, execution)
        .expect("fixture read request remains live")
        .expect("the changed V2 successor is readable");
    let successor_cargo = successor_view
        .entities()
        .iter()
        .find(|record| record.entity_id == cargo_id)
        .expect("the successor retains the cargo identity");
    assert_ne!(
        successor_cargo.authoritative_aspect_state, entity.authoritative_aspect_state,
        "the successor must diverge so latest-root leakage is observable"
    );
    let context = world
        .runtime
        .read_truth()
        .query_plan_context(&transitioned.snapshot)
        .expect("the retained V2 snapshot remains plan-admissible");

    let explicit = PlannedQueryPacket::explicit_targets(
        "retained-v2-explicit-targets",
        context.clone(),
        vec![
            RecordRef::Entity(entity.entity_id),
            RecordRef::Relation(relation.relation_id),
        ],
    );
    let kind_scan = PlannedQueryPacket {
        label: "retained-v2-kind-scan".to_owned(),
        context_id: context.clone(),
        scope: QueryScope::EntityKindScan {
            kind_id: entity.kind.kind_id,
            partition_scope: Some(std::sync::Arc::from([entity.entity_id.partition_id])),
        },
        locality: QueryLocalityClass::PartitionBounded {
            partitions: std::sync::Arc::from([entity.entity_id.partition_id]),
        },
        ordering: QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_171_201),
        target_count_hint: 1,
    };
    let traversal = PlannedQueryPacket {
        label: "retained-v2-traversal".to_owned(),
        context_id: context,
        scope: QueryScope::OutgoingNeighborhood {
            seeds: std::sync::Arc::from([relation.source]),
            relation_kind_scope: None,
        },
        locality: QueryLocalityClass::CrossPartitionTraversal,
        ordering: QueryOrderingContract::CanonicalTraversalOrder,
        access_contract: QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: QueryExecutionShape::BulkPacketized,
        reduction: ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_171_202),
        target_count_hint: 1,
    };

    let explicit_outcome = execute_exact_query(execution, &world, &transitioned.snapshot, explicit);
    assert_eq!(explicit_outcome.result.entities, vec![entity.clone()]);
    assert_eq!(explicit_outcome.result.relations, vec![relation.clone()]);
    assert_matches_retained_root(&explicit_outcome, &retained_entities, &retained_relations);
    assert_schema_v2(&explicit_outcome);

    let kind_outcome = execute_exact_query(execution, &world, &transitioned.snapshot, kind_scan);
    assert_eq!(kind_outcome.result.entities, expected_kind_entities);
    assert_matches_retained_root(&kind_outcome, &retained_entities, &retained_relations);
    assert_schema_v2(&kind_outcome);

    let traversal_outcome =
        execute_exact_query(execution, &world, &transitioned.snapshot, traversal);
    let traversed_relation = traversal_outcome
        .result
        .relations
        .iter()
        .find(|record| record.relation_id == relation.relation_id)
        .expect("the retained traversal returns the exact route relation");
    assert_eq!(traversed_relation.source, relation.source);
    assert_eq!(traversed_relation.target, relation.target);
    assert_matches_retained_root(&traversal_outcome, &retained_entities, &retained_relations);
    assert_schema_v2(&traversal_outcome);

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

fn execute_exact_query(
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

fn assert_schema_v2(outcome: &worth_relational::facade::query::QueryExecutionOutcome) {
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

fn assert_matches_retained_root(
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
