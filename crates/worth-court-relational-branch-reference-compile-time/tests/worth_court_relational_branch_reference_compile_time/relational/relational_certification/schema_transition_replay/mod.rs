#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_relational::facade::query::DeterministicQueryPlanKey, worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::QueryAccessContract, worth_relational::facade::query::QueryExecutionShape, worth_relational::facade::query::QueryLocalityClass, worth_relational::facade::query::QueryOrderingContract, worth_relational::facade::query::QueryScope, worth_relational::facade::transactions::RecordRef, worth_relational::facade::query::ReductionDiscipline, worth_relational::facade::replay::RelationalReplayRequest, worth_relational::facade::replay::ReplayExecutionMode, worth_relational::facade::replay::ReplayVerificationMode, worth_relational::facade::schema::SchemaVersionId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_relational::facade::transactions::WorkerIntentBatch, crate::relational_certification::ordinary_context::schema_transition_replay::assert_matches_retained_root, crate::relational_certification::ordinary_context::schema_transition_replay::assert_schema_v2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, crate::relational_certification::ordinary_context::schema_transition_replay::execute_exact_query, worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2 as lower_hazard_v2_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot};

#[test]
fn retained_v2_snapshot_materializes_kind_metadata_from_its_root_authority() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = BranchId("main".to_owned());
    let transitioned = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
        lower_hazard_v2_batch(&world.handles).unwrap(),
    );
    let successor = commit_branch_batch_with_result(
        execution,
        &world.runtime,
        branch,
        worth_relational::facade::transactions::WorkerIntentBatch::new("supply-chain-v2-successor"),
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
