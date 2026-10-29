#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::history::BranchId, worth_relational::facade::publication::PatchStreamRequest, worth_relational::facade::mvcc::RelationalInterruptionBoundary, worth_relational::facade::inspection::RelationalMvccCostScope, worth_relational::facade::mvcc::RelationalOperationControl, worth_relational::facade::mvcc::RelationalOperationInterruption, worth_relational::facade::mvcc::RelationalPublicationOutcome, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenarios::version_boundary::hazard_v2_transition, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2 as lower_hazard_v2_batch};

#[test]
fn cancelled_schema_transition_leaves_no_target_or_branch_residue() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(request_execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = BranchId("hazard-v2".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    world
        .runtime
        .fork_branch(request_execution, branch.clone(), source)
        .unwrap();
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let before = world.runtime.branch_reference_state(&branch).unwrap();
    let catalog_before = world.runtime.history().immutable_commit_count();
    let stream_before = world
        .runtime
        .publication()
        .read_patch_stream(
            request_execution,
            worth_relational::facade::publication::PatchStreamRequest::default(),
        )
        .unwrap();
    let replay_before = world.runtime.publication().latest_replay();
    let cost_scope = worth_relational::facade::inspection::RelationalMvccCostScope::capture(
        &world.runtime,
        vec![identity],
    );
    let control = worth_relational::facade::mvcc::RelationalOperationControl::uninterrupted()
        .with_injected_interruption_for_test(
            worth_relational::facade::mvcc::RelationalInterruptionBoundary::BeforeCriticalSection,
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled,
            std::num::NonZeroUsize::MIN,
        );
    let mut transaction = world
        .runtime
        .begin_branch_schema_transition_with_control(
            &basis,
            hazard_v2_transition(),
            None,
            world
                .program
                .schema_registry_for_version(worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion::V2)
                .unwrap(),
            control,
        )
        .unwrap();
    transaction
        .push_batch(
            lower_hazard_v2_batch(&world.handles).unwrap(),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_branch_transaction(request_execution, transaction)
        .unwrap();
    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(request_execution, candidate),
        worth_relational::facade::mvcc::RelationalPublicationOutcome::Interrupted(_)
    ));
    assert_eq!(
        world.runtime.branch_reference_state(&branch).unwrap(),
        before
    );
    assert_eq!(
        world.runtime.history().immutable_commit_count(),
        catalog_before
    );
    assert_eq!(
        world
            .runtime
            .publication()
            .read_patch_stream(
                request_execution,
                worth_relational::facade::publication::PatchStreamRequest::default()
            )
            .unwrap(),
        stream_before
    );
    assert_eq!(world.runtime.publication().latest_replay(), replay_before);
    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().candidate_releases, 1);

    let ordinary = world
        .runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("cancelled target admission changed no runtime-global schema state");
    drop(ordinary);
}
