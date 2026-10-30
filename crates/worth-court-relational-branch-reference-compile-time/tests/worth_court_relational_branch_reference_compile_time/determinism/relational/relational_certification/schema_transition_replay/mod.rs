#[allow(unused_imports)]
pub(crate) use {worth_relational::facade::history::BranchId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId, worth_relational::facade::query::DeterministicQueryPlanKey, worth_relational::facade::query::PlannedQueryPacket, worth_relational::facade::query::QueryAccessContract, worth_relational::facade::query::QueryExecutionShape, worth_relational::facade::query::QueryLocalityClass, worth_relational::facade::query::QueryOrderingContract, worth_relational::facade::query::QueryScope, worth_relational::facade::transactions::RecordRef, worth_relational::facade::query::ReductionDiscipline, worth_relational::facade::replay::RelationalReplayRequest, worth_relational::facade::replay::ReplayExecutionMode, worth_relational::facade::replay::ReplayVerificationMode, worth_relational::facade::schema::SchemaVersionId, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale, worth_relational::facade::transactions::WorkerIntentBatch, crate::relational_certification::ordinary_context::schema_transition_replay::assert_matches_retained_root, crate::relational_certification::ordinary_context::schema_transition_replay::assert_schema_v2, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::certified_baseline::certified_supply_chain_world, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_branch_batch_with_result, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::commit_supply_chain_delta, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::comparison::compare, crate::relational_certification::ordinary_context::schema_transition_replay::execute_exact_query, worth_testkit_relational::relational_certification::ordinary_context::invariant_oracle_expectations::expected_supply_chain_branch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::branch_operations::fork_supply_chain_branch_from_main, crate::relational_certification::ordinary_context::world::supply_chain::runtime_driver::footprint_batch::lower_cargo_footprint_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::runtime_driver::delta_batches::lower_hazard_v2 as lower_hazard_v2_batch, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::observation::observe_snapshot as observe_supply_chain_snapshot};

#[test]
fn v2_latest_replay_matches_the_canonical_envelope_and_reconstructs() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (world, _) = certified_supply_chain_world(execution, worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scale::SupplyChainScale::court());
    let branch = BranchId("main".to_owned());
    let batch = lower_hazard_v2_batch(&world.handles).unwrap();
    let committed = commit_supply_chain_delta(
        execution,
        &world.runtime,
        &world.program,
        branch.clone(),
        worth_testkit_relational::relational_certification::ordinary_context::world::supply_chain::scenario_delta_vocabulary::DeltaId::AdoptHazardClassificationV2,
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
                execution_mode: worth_relational::facade::replay::ReplayExecutionMode::SerialDeterministic,
                verification_mode: worth_relational::facade::replay::ReplayVerificationMode::NormalRecoveryVerification,
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
