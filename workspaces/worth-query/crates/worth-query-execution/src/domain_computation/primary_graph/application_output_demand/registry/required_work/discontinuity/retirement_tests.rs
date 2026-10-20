//! Creation prepays retirement even after other genuine occurrences grow the tree.
use super::*;
use crate::domain_computation::execution_runtime::product_world::{
    test_product_world_resources, WorthQueryProductWorldResources,
};
use std::sync::atomic::Ordering;

fn meter() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
        maximum_work_visits: None,
        maximum_preparation_bytes: None,
    })
}

#[test]
fn issued_cursor_retirement_remains_paid_after_native_occurrence_growth() {
    let (_, clock, invalidation, execution) = test_product_world_resources().into_parts();
    let configured = WorthQueryProductWorldResources::new(
        worth_runtime_world::facade::RuntimeWorldBudgets::without_policy_limits(),
        clock,
        invalidation,
        execution.policy,
    );
    let configured = match execution.authority {
        Some(authority) => configured.with_execution_authority(authority),
        None => configured,
    };
    let world = crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources(configured);
    let root = world.application.current_world();
    let product = world
        .application
        .product_runtime
        .admit_product_occurrence(root.occurrence())
        .unwrap();
    let original = product.observation().lifecycle_incarnation();
    let branch = BranchId("main".into());
    let epoch = CommitId(1);
    let mut state = DemandRegistryState::default();
    let mut first = meter();
    state
        .page_required_discontinuity(original, &branch, epoch, &mut first)
        .unwrap();
    let exact_one = state
        .required_custody_retained_bytes
        .load(Ordering::Acquire);
    // A separately installed real one-cursor policy isolates creation's fixed
    // work from its different, correctly narrower lifetime retirement quote.
    let mut finite = DemandRegistryState::default();
    finite.required_budget_bytes = Some(exact_one);
    let mut finite_creation = meter();
    finite
        .page_required_discontinuity(original, &branch, epoch, &mut finite_creation)
        .unwrap();
    let mut finite_retirement = meter();
    charge_outer_retirement(&mut finite_retirement, Some(exact_one)).unwrap();
    let prepaid = first
        .charged_work()
        .checked_sub(finite_creation.charged_work())
        .unwrap()
        + finite_retirement.charged_work();
    let mut created = Vec::new();
    for _ in 0..24 {
        let fork = world
            .application
            .with_host_advancement(|phase| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| {
                        components.fork_relational().reuse_exact_signal_basis()
                    })
                    .create(&phase)
                    .unwrap()
            })
            .unwrap();
        let product = world
            .application
            .product_runtime
            .admit_product_occurrence(fork.occurrence())
            .unwrap();
        let occurrence = product.observation().lifecycle_incarnation();
        state
            .page_required_discontinuity(occurrence, &branch, epoch, &mut meter())
            .unwrap();
        created.push((fork, product));
    }
    assert_eq!(
        state.discontinuity_cursors.occurrences.len(),
        created.len() + 1
    );
    let mut grown_retirement = meter();
    charge_outer_retirement(
        &mut grown_retirement,
        Some(
            state
                .required_custody_retained_bytes
                .load(Ordering::Acquire),
        ),
    )
    .unwrap();
    assert!(
        prepaid >= grown_retirement.charged_work(),
        "issued prepayment {prepaid} does not cover grown retirement {}",
        grown_retirement.charged_work()
    );
    drop(state.take_discontinuity_cursors(original).unwrap());
    assert!(!state
        .discontinuity_cursors
        .occurrences
        .contains_key(&original));
    let ledger = Arc::clone(&state.required_custody_retained_bytes);
    drop(state);
    assert_eq!(ledger.load(Ordering::Acquire), 0);
    drop(finite);
    drop(product);
    for (fork, product) in created {
        drop(product);
        world
            .application
            .with_host_advancement(|phase| {
                assert!(world
                    .application
                    .on_branch(fork)
                    .close(&phase)
                    .unwrap()
                    .is_complete());
            })
            .unwrap();
    }
}
