#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::owner_service_completion::healthy_bundle::*;

#[test]
fn discarded_candidate_leaves_truth_unchanged_and_releases_its_obligation() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use super::super::invariant_oracle_expectations::expected_supply_chain_branch;
    use super::super::world::supply_chain::{
        certified_supply_chain_world, compare, lower_supply_chain_production_delta,
        observe_supply_chain_observation, BranchLabel, DeltaId, SupplyChainScale,
    };

    let (world, _baseline) = certified_supply_chain_world(execution, SupplyChainScale::court());
    let services = world.runtime.owner_component_services();
    let (_, source) = services
        .fork_port()
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("the non-empty Supply Chain main branch is a fork source");
    let identity = services
        .fork_port()
        .fork_branch(execution, BranchId("storm".to_owned()), source)
        .expect("the fork port installs the exact storm branch")
        .target_identity()
        .clone();
    let (descriptor, basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("the basis port observes the storm predecessor");
    let batch = lower_supply_chain_production_delta(
        execution,
        &world.runtime,
        &world.program,
        &world.handles,
        identity.branch_id(),
        &BTreeSet::new(),
        DeltaId::StormRerouteAurora,
    )
    .expect("the real storm branch admits its semantic reroute delta");
    let mut transaction = world
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("the exact storm predecessor opens a transaction");
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the semantic reroute remains inside its declared budget");
    let candidate = services
        .preparation_port()
        .prepare_branch_transaction(execution, transaction)
        .expect("the preparation port installs the canonical candidate");
    let discarded = services
        .preparation_port()
        .discard_prepared_candidate(candidate)
        .expect("the preparation port consumes the candidate without movement");
    assert_eq!(discarded.branch(), identity.branch_id());
    assert_eq!(
        discarded.released_record_reservation_count(),
        0,
        "the update-only Supply Chain delta must not invent record reservations"
    );

    let (after_descriptor, after_basis) = services
        .basis_port()
        .observe_branch(&identity)
        .expect("discard leaves the predecessor observable");
    assert_eq!(after_descriptor, descriptor);
    let observation = after_basis.observation();
    let handles = world.handles.for_observation(&observation);
    let observed = observe_supply_chain_observation(
        execution,
        &world.program,
        &handles,
        &world.runtime,
        &observation,
    )
    .expect("discarded work leaves neutral Supply Chain truth observable");
    compare(
        &expected_supply_chain_branch(&world.program, BranchLabel::Storm, None),
        &observed,
    )
    .expect("discarded work leaves the independent baseline unchanged");
    assert!(matches!(
        services.lifecycle_port().delete_branch(execution, &identity),
        Ok(RelationalBranchDeletionOutcome::Deleted(ref deleted))
            if deleted.identity() == &identity
    ));
}
