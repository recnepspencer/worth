#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::branch::RelationalBranchBasisDenial, worth_relational::facade::branch::RelationalBranchLifecyclePosture, worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial, worth_relational::facade::branch::RelationalForkDenial, worth_relational::facade::inspection::RelationalMvccCostScope, worth_relational::facade::mvcc::RelationalPublicationDenial, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, crate::relational_certification::ordinary_context::mvcc_branch_fork_fixture::fork_branch, crate::relational_certification::ordinary_context::mvcc_archive_retention::snapshot_contains_entity};

#[test]
fn archive_preserves_existing_reads_and_closes_every_new_mutation_door() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let identity = fork_branch(execution, &world.runtime, "storm");
    let (_, basis) = world.runtime.observe_branch(&identity).unwrap();
    let immutable_observation = basis.observation();
    let retained_commit = immutable_observation.canonical_commit().cloned();
    let retained_index = immutable_observation.correctness_index();
    let retained_root = immutable_observation.selected_root_identity();
    let snapshot = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&immutable_observation)
        .unwrap();
    let voyage_id = world.handles.aurora_voyage().id;
    assert!(snapshot_contains_entity(
        execution,
        &world.runtime,
        &snapshot,
        voyage_id
    ));
    let external = world.runtime.retain_component_basis(&basis).unwrap();
    let active = world
        .runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate_transaction = world
        .runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(execution, candidate_transaction)
        .unwrap();
    let cost_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(&world.runtime, vec![identity.clone()]);

    world.runtime.archive_branch(execution, &identity).unwrap();
    assert_eq!(basis.observation().selected_root_identity(), retained_root);
    assert_eq!(
        immutable_observation.canonical_commit(),
        retained_commit.as_ref()
    );
    assert_eq!(immutable_observation.correctness_index(), retained_index);
    assert!(snapshot_contains_entity(
        execution,
        &world.runtime,
        &snapshot,
        voyage_id
    ));
    assert_eq!(
        world
            .runtime
            .branch_reference_state(identity.branch_id())
            .unwrap()
            .lifecycle_posture(),
        worth_relational::facade::branch::RelationalBranchLifecyclePosture::Archived,
    );
    assert!(matches!(
        world.runtime.observe_branch(&identity),
        Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(_))
    ));
    assert!(matches!(
        world.runtime.observe_fork_source(identity.branch_id()),
        Err(worth_relational::facade::branch::RelationalForkDenial::SourceArchived)
    ));
    assert!(matches!(
        world
            .runtime
            .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary()),
        Err(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial::Archived)
    ));
    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(execution, candidate),
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Denied(worth_relational::facade::mvcc::RelationalPublicationDenial::Archived)
    ));
    drop(active);
    world.runtime.release_component_basis(external).unwrap();
    world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();

    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().candidate_releases, 1);
    assert_eq!(cost.retention_cost_delta().transaction_releases, 1);
    assert_eq!(cost.retention_cost_delta().external_pin_releases, 1);
}
