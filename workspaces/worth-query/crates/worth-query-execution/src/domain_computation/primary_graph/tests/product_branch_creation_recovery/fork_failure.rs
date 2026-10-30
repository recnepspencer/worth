//! Genuine two-owner creation with an occupied Signal destination after the Relational fork.
use super::super::fixture::AuthorizationWorld;
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchCreateError,
    WorthQueryProductBranchCreationRecovery,
};
use worth_runtime_world::facade::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, RelationalBranchCreationPlan,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationSource, SignalBranchCreationPlan,
};

pub(super) fn create(
    world: &AuthorizationWorld,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryProductBranchCreationRecovery,
    WorthQueryProductBranch,
) {
    let root = world.application.current_world();
    let source = world
        .application
        .product_runtime
        .admit_product_occurrence(root.occurrence())
        .expect("the root occurrence remains selectable");
    let occupied_signal =
        worth_signal::facade::branch::validate_signal_branch_name("query-product-1-signal")
            .expect("the public builder's first Signal destination is valid");
    let intent = ProductBranchCreationIntent::from_source(
        "occupied-signal-destination",
        ProductBranchCreationPlans::new(
            RelationalBranchCreationPlan::ReuseExact,
            SignalBranchCreationPlan::ForkExact {
                target: occupied_signal,
            },
        ),
    )
    .expect("the owner-driven collision fixture is valid");
    let cancellation = RuntimeWorldCancellationSource::new();
    let RuntimeWorldBranchCreationOutcome::Performed(occupied) = world
        .application
        .product_runtime
        .create_product_branch(execution, &source, None, intent, &cancellation.token())
        .expect("the real Signal destination is occupied through World")
    else {
        panic!("the collision fixture must publish its Signal fork");
    };
    drop(source);
    let occupied_branch =
        WorthQueryProductBranch::from_occurrence(occupied.lifecycle_incarnation());
    drop(occupied);
    let error = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(root)
                .components(|parts| parts.fork_relational().fork_signal())
                .create(&phase)
                .expect_err("the occupied Signal sibling retains the real Relational fork")
        })
        .expect("the fixture read owns its installed host call");
    let WorthQueryProductBranchCreateError::ProductUnpublished(recovery) = error else {
        panic!("the public builder preserves Query-owned recovery custody: {error:?}");
    };
    (recovery, occupied_branch)
}
