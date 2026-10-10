//! The readiness owner never grants a second attempt to rescue optional capacity.
use super::*;
use crate::domain_computation::{
    execution_runtime::WorthQueryInvalidationResourceDenial,
    primary_graph::tests::fixture::installed_authorization_world_with_product_resources,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

#[test]
fn refused_registration_capacity_does_not_select_a_member_twice_in_one_advance() {
    let product_resources =
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources();
    let resources = product_resources.invalidation_resources();
    let world = installed_authorization_world_with_product_resources(product_resources);
    let runtime = &world.application;
    let mut admission = runtime.demand_request_admission();
    let (shared, positioned) = super::super::selection::select_required_basis(
        runtime,
        runtime.current_world(),
        &mut admission,
    )
    .unwrap();
    let input = SelectedDecisionInput {
        shared: &shared,
        positioned: &positioned,
        runtime,
    };
    let source = WorthQueryObservedSourceEpoch::new(
        [1; 32],
        [2; 32],
        EntityId::new(PartitionId::main(), 1, 1),
        shared
            .selected()
            .product()
            .observation()
            .lifecycle_incarnation(),
        0,
        [3; 32],
    );
    let mut performed = PerformedMembers::start();
    let family = TypeId::of::<()>();
    let mut attempts = 0;
    if performed
        .fresh(family, source.clone(), &input, &mut admission)
        .unwrap_or_else(|_| panic!("Fresh readiness comparison is admitted"))
        .is_some()
    {
        attempts += 1;
        // The effect has completed before optional registration. The wave's
        // mark changes only this state; selection alone did not perform it.
        performed.entries[0].performed = true;
        // Optional registration draws from this actual shared index ledger.
        // Exhaust it after selection, then return its exact capacity refusal.
        let maximum = resources.installation().maximum_retained_bytes;
        let held = resources
            .reserve_retained_capacity(maximum - resources.retained_capacity_bytes())
            .unwrap();
        assert_eq!(
            resources.reserve_retained_capacity(1).err(),
            Some(
                WorthQueryInvalidationResourceDenial::RetentionCapacityExhausted {
                    requested: 1,
                    retained: maximum,
                    maximum,
                }
            )
        );
        drop(held);
    }
    if performed
        .fresh(family, source.clone(), &input, &mut admission)
        .unwrap_or_else(|_| panic!("Fresh readiness comparison is admitted"))
        .is_some()
    {
        attempts += 1;
    }
    assert_eq!(
        attempts, 1,
        "returning optional capacity cannot grant a second attempt"
    );
    // A later advancement has a new record and may try this member again.
    assert!(PerformedMembers::start()
        .fresh(family, source, &input, &mut admission)
        .unwrap_or_else(|_| panic!("later advancement readiness is admitted"))
        .is_some());
}
