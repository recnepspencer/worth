use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchCreationOutcome,
    RuntimeWorldCancellationSource, RuntimeWorldServiceDenial, SignalBranchCreationPlan,
};

use super::capacity_evidence_tests::capacity_resources;
use crate::basis::{
    WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationRecoveryCause,
};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCloseScope, WorthQueryProductBranchCreationDenial,
};
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
fn unpublished_creation_refunds_activation_and_world_slot_before_recovery_release() {
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
    // Only root + occupied retain slots. Recovery retains component custody,
    // not either branch-slot reservation.
    let next = runtime
        .reserve_product_branch_creation(&source, None)
        .expect("ProductUnpublished must refund its Query activation reservation");
    drop(next);
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
    let probe = fork().expect("unpublished recovery has released both branch-slot reservations");
    let probe_lease = runtime
        .admit_product_occurrence(probe.occurrence())
        .unwrap();
    assert_eq!(
        probe_lease.branch_identity().name().as_str(),
        "query-product-2"
    );
    drop(probe_lease);
    assert!(matches!(
        fork(),
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted {
                limit: 3,
                occupancy: 3
            }
        ))
    ));
    runtime
        .begin_product_branch_close(probe, WorthQueryProductBranchCloseScope::Workspace)
        .unwrap()
        .finish()
        .unwrap();
    // A real holder below Query fills World first. It uses the production World
    // port and never acquires a Query activation gate or a second policy.
    let outside_intent = ProductBranchCreationIntent::from_source(
        "outside-world-holder",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(outside) = runtime
        .owner
        .branch_port()
        .create_product_branch(
            source.observation().clone(),
            outside_intent,
            &cancellation.token(),
        )
        .unwrap()
    else {
        panic!("the outside World holder reuses two real admitted components")
    };
    for _ in 0..2 {
        assert_eq!(world_evidence(fork().unwrap_err()), (3, 3));
    }
    let release = recovery.release_obligations(0).unwrap();
    assert_eq!(release.retired_component_count(), 1);
    assert_eq!(world_evidence(fork().unwrap_err()), (3, 3));
    let retirement = runtime
        .owner
        .branch_port()
        .retire_product_branch(&outside)
        .unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    drop(retirement);
    drop(outside);
    let replacement = fork().expect("retiring the outside holder returns the World slot");
    let replacement = runtime
        .admit_product_occurrence(replacement.occurrence())
        .unwrap();
    // Unpublished used 1, probe used 2; World refusals used 3, 4 and 5. Coordination refusals
    // precede naming, but every World intent already owns its destination name.
    assert_eq!(
        replacement.branch_identity().name().as_str(),
        "query-product-6"
    );
    assert!(matches!(
        fork(),
        Err(WorthQueryProductBranchCreateError::Creation(
            WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted {
                limit: 3,
                occupancy: 3
            }
        ))
    ));
    drop(replacement);
    drop((occupied, source));
}

fn world_evidence(error: WorthQueryProductBranchCreateError) -> (usize, usize) {
    let WorthQueryProductBranchCreateError::Creation(WorthQueryProductBranchCreationDenial::World(
        RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::BranchCapacityExhausted { limit, occupancy },
        ),
    )) = error
    else {
        panic!("expected exact World membership evidence, got {error:?}")
    };
    (limit, occupancy)
}
