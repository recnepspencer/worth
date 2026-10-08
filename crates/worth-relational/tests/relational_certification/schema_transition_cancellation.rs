use super::world::supply_chain::{
    certified_supply_chain_world, hazard_v2_transition, lower_hazard_v2_batch, SchemaVersion,
    SupplyChainScale,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::RelationalMvccCostScope;
use worth_relational::facade::mvcc::{
    RelationalInterruptionBoundary, RelationalOperationControl, RelationalOperationInterruption,
    RelationalPublicationOutcome, RelationalTransactionIntent,
};
use worth_relational::facade::publication::PatchStreamRequest;

#[test]
fn cancelled_schema_transition_leaves_no_target_or_branch_residue() {
    let (world, _) = certified_supply_chain_world(SupplyChainScale::court());
    let branch = BranchId("hazard-v2".to_owned());
    let (_, source) = world
        .runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .unwrap();
    world.runtime.fork_branch(branch.clone(), source).unwrap();
    let identity = world.runtime.branch_identity(&branch).unwrap();
    let basis = world.runtime.admit_branch_basis(&identity).unwrap();
    let before = world.runtime.branch_reference_state(&branch).unwrap();
    let catalog_before = world.runtime.history().immutable_commit_count();
    let stream_before = world
        .runtime
        .publication()
        .read_patch_stream(PatchStreamRequest::default())
        .unwrap();
    let replay_before = world.runtime.publication().latest_replay();
    let cost_scope = RelationalMvccCostScope::capture(&world.runtime, vec![identity]);
    let control = RelationalOperationControl::uninterrupted().with_injected_interruption(
        RelationalInterruptionBoundary::BeforeCriticalSection,
        RelationalOperationInterruption::Cancelled,
        1,
    );
    let mut transaction = world
        .runtime
        .begin_branch_schema_transition_with_control(
            &basis,
            hazard_v2_transition(),
            None,
            world
                .program
                .schema_registry_for_version(SchemaVersion::V2)
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
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        world
            .runtime
            .publication_port()
            .compare_and_publish(candidate),
        RelationalPublicationOutcome::Interrupted(_)
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
            .read_patch_stream(PatchStreamRequest::default())
            .unwrap(),
        stream_before
    );
    assert_eq!(world.runtime.publication().latest_replay(), replay_before);
    let cost = world.runtime.observe_mvcc_counters(&cost_scope).unwrap();
    assert_eq!(cost.retention_cost_delta().candidate_releases, 1);

    let ordinary = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("cancelled target admission changed no runtime-global schema state");
    drop(ordinary);
}
