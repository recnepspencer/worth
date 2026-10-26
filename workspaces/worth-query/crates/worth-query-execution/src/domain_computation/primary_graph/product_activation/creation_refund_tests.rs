use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchCreationOutcome,
    RuntimeWorldCancellationSource, RuntimeWorldServiceDenial, SignalBranchCreationPlan,
};

use super::capacity_evidence_tests::capacity_resources;
use crate::basis::{
    WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationRecoveryCause,
};
use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchCreationDenial;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources;

#[test]
fn world_denial_refunds_creation_activation_before_the_next_fork() {
    let world = installed_authorization_world_with_product_resources(capacity_resources(3));
    let runtime = world.application.product_runtime();
    let root = runtime.root_product_branch();
    let fork = || {
        world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| {
                        components
                            .reuse_exact_relational_basis()
                            .reuse_exact_signal_basis()
                    })
                    .create(&phase)
            })
            .unwrap()
    };
    let first = fork().unwrap();
    // Root + first is L-1. World must refuse after Query admitted its last slot.
    let source = runtime.admit_product_occurrence(root.occurrence()).unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    cancellation.cancel();
    let intent = ProductBranchCreationIntent::from_source(
        "cancelled-fork",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    assert!(matches!(
        runtime.create_product_branch(&source, None, intent, &cancellation.token()),
        Err(WorthQueryProductBranchCreationDenial::World(
            RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
            )
        ))
    ));
    drop(source);
    let last = fork().expect("the World denial must refund Query activation custody");
    assert_ne!(first, last);
    assert!(matches!(
        fork(),
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted {
                limit: 3,
                occupancy: 3
            }
        ))
    ));
}

#[test]
fn unpublished_creation_refunds_activation_while_world_recovery_holds_its_slot() {
    let world = installed_authorization_world_with_product_resources(capacity_resources(3));
    let runtime = world.application.product_runtime();
    let root = runtime.root_product_branch();
    let source = runtime.admit_product_occurrence(root.occurrence()).unwrap();
    let intent = ProductBranchCreationIntent::from_source(
        "occupied-signal-destination",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ForkExact {
                target: worth_signal::facade::branch::validate_signal_branch_name(
                    "query-product-1-signal",
                )
                .unwrap(),
            },
        ),
    )
    .unwrap();
    let cancellation = RuntimeWorldCancellationSource::new();
    let RuntimeWorldBranchCreationOutcome::Performed(occupied) = runtime
        .create_product_branch(&source, None, intent, &cancellation.token())
        .unwrap()
    else {
        panic!("the real Signal collision fixture must publish")
    };
    let error = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(root)
                .components(|components| components.fork_relational().fork_signal())
                .create(&phase)
        })
        .unwrap()
        .unwrap_err();
    let WorthQueryProductBranchCreateError::ProductUnpublished(recovery) = error else {
        panic!("the real Signal denial must retain its Relational sibling: {error:?}")
    };
    let inspection = recovery.inspect().unwrap();
    assert_eq!(
        inspection.cause(),
        WorthQueryProductBranchCreationRecoveryCause::SiblingOwnerDenied
    );
    assert_eq!(inspection.owner_effect_count(), 1);
    drop(inspection);
    // Only root + occupied retain Query slots. Recovery owns World custody,
    // not an activation gate, so the third Query reservation is available.
    let next = runtime
        .reserve_product_branch_creation(&source, None)
        .expect("ProductUnpublished must refund its Query activation reservation");
    drop(next);
    let release = recovery.release_obligations(0).unwrap();
    assert_eq!(release.retired_component_count(), 1);
    drop((occupied, source));
}
