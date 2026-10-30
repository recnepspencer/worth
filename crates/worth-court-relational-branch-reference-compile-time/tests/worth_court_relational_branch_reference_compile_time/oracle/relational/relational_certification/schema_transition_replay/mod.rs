#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_relational::facade::query::DeterministicQueryPlanKey, worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::QueryAccessContract, worth_relational::facade::query::QueryExecutionShape, worth_relational::facade::query::QueryLocalityClass, worth_relational::facade::query::QueryOrderingContract, worth_relational::facade::query::QueryScope, worth_relational::facade::transactions::RecordRef, worth_relational::facade::query::ReductionDiscipline, worth_relational::facade::replay::RelationalReplayRequest, worth_relational::facade::replay::ReplayExecutionMode, worth_relational::facade::replay::ReplayVerificationMode, worth_relational::facade::schema::SchemaVersionId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_relational::facade::transactions::WorkerIntentBatch, crate::relational_certification::ordinary_context::schema_transition_replay::assert_matches_retained_root, crate::relational_certification::ordinary_context::schema_transition_replay::assert_schema_v2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, crate::relational_certification::ordinary_context::schema_transition_replay::execute_exact_query, worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2 as lower_hazard_v2_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot};

#[test]
fn retained_v2_query_plans_materialize_from_their_exact_root_authority() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = BranchId("hazard-v2".to_owned());
    fork_supply_chain_branch_from_main(execution, &world.runtime, branch.clone());
    let transitioned = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
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
            worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::semantic_key::BranchLabel::HazardV2,
            Some(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2),
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
        lower_cargo_footprint_batch(&world.handles, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court(), 1),
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

    let explicit = worth_relational::facade::query::PlannedQueryPacket::explicit_targets(
        "retained-v2-explicit-targets",
        context.clone(),
        vec![
            worth_relational::facade::transactions::RecordRef::Entity(entity.entity_id),
            worth_relational::facade::transactions::RecordRef::Relation(relation.relation_id),
        ],
    );
    let kind_scan = PlannedQueryPacket {
        label: "retained-v2-kind-scan".to_owned(),
        context_id: context.clone(),
        scope: worth_relational::facade::query::QueryScope::EntityKindScan {
            kind_id: entity.kind.kind_id,
            partition_scope: Some(std::sync::Arc::from([entity.entity_id.partition_id])),
        },
        locality: worth_relational::facade::query::QueryLocalityClass::PartitionBounded {
            partitions: std::sync::Arc::from([entity.entity_id.partition_id]),
        },
        ordering: worth_relational::facade::query::QueryOrderingContract::CanonicalEntityIdOrder,
        access_contract:
            worth_relational::facade::query::QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: worth_relational::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: worth_relational::facade::query::ReductionDiscipline::DeterministicMerge,
        plan_key: DeterministicQueryPlanKey(9_171_201),
        target_count_hint: 1,
    };
    let traversal = PlannedQueryPacket {
        label: "retained-v2-traversal".to_owned(),
        context_id: context,
        scope: worth_relational::facade::query::QueryScope::OutgoingNeighborhood {
            seeds: std::sync::Arc::from([relation.source]),
            relation_kind_scope: None,
        },
        locality: worth_relational::facade::query::QueryLocalityClass::CrossPartitionTraversal,
        ordering: worth_relational::facade::query::QueryOrderingContract::CanonicalTraversalOrder,
        access_contract:
            worth_relational::facade::query::QueryAccessContract::AuthoritativeStorageOnly,
        execution_shape: worth_relational::facade::query::QueryExecutionShape::BulkPacketized,
        reduction: worth_relational::facade::query::ReductionDiscipline::DeterministicMerge,
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
