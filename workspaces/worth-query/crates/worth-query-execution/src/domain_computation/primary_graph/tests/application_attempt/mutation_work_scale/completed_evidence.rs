use std::num::NonZeroUsize;

use super::{idempotency, no_demand_mutation_program, WorthQueryApplicationCommitOutcome};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_completed_evidence_capacity;

fn world_with_capacity(
    maximum: usize,
) -> crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld {
    installed_authorization_world_with_completed_evidence_capacity(
        NonZeroUsize::new(maximum).expect("a fixture must install finite capacity"),
    )
}

#[test]
fn completed_evidence_capacity_denies_before_publication_and_refunds_after_last_observer() {
    let baseline = world_with_capacity(4 * 1_024 * 1_024);
    let baseline_capacity = baseline
        .application
        .primary_provider
        .completed_evidence_capacity_for_test();
    let before = baseline_capacity.retained_bytes();
    let committed = baseline.application.compare_and_commit_application(
        no_demand_mutation_program(&baseline, false),
        idempotency(91, 92),
    );
    let WorthQueryApplicationCommitOutcome::Committed(baseline_receipt) = committed else {
        panic!("the funded actual mutation must publish");
    };
    let actual_backing = baseline_capacity.retained_bytes() - before;
    assert!(actual_backing > 1);
    assert!(!baseline_receipt
        .mutation_work()
        .unwrap()
        .touched_records()
        .is_empty());
    drop(baseline_receipt);
    drop(baseline);

    let denied = world_with_capacity(before + actual_backing - 1);
    let denied_capacity = denied
        .application
        .primary_provider
        .completed_evidence_capacity_for_test();
    assert_eq!(denied_capacity.retained_bytes(), before);
    let product_before = denied
        .selected_product()
        .product()
        .selected_commit()
        .clone();
    let relational_before = denied
        .selected_product()
        .product()
        .relational_basis()
        .observation()
        .commit_receipt()
        .cloned();
    let denied_result = denied.application.compare_and_commit_application(
        no_demand_mutation_program(&denied, false),
        idempotency(91, 92),
    );
    let WorthQueryApplicationCommitOutcome::Deferred(deferred) = denied_result else {
        panic!("completed evidence exhaustion must retain typed retry posture");
    };
    assert_eq!(
        deferred.kind(),
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitDeferredKind::RetentionCapacityExhausted
    );
    assert_eq!(
        deferred.stage(),
        crate::domain_computation::provider_session::WorthQueryProviderSessionProtocolStage::Commit
    );
    assert_eq!(denied_capacity.retained_bytes(), before);
    assert_eq!(
        denied.selected_product().product().selected_commit(),
        &product_before
    );
    assert_eq!(
        denied
            .selected_product()
            .product()
            .relational_basis()
            .observation()
            .commit_receipt()
            .cloned(),
        relational_before
    );

    let exact = world_with_capacity(before + actual_backing);
    let exact_capacity = exact
        .application
        .primary_provider
        .completed_evidence_capacity_for_test();
    let published = exact.application.compare_and_commit_application(
        no_demand_mutation_program(&exact, false),
        idempotency(91, 92),
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = published else {
        panic!("the exact real backing must publish");
    };
    let observed = receipt.mutation_work().unwrap().clone();
    assert_eq!(exact_capacity.retained_bytes(), before + actual_backing);
    drop(receipt);
    drop(exact);
    assert_eq!(exact_capacity.retained_bytes(), actual_backing);
    drop(observed);
    assert_eq!(exact_capacity.retained_bytes(), 0);
}
