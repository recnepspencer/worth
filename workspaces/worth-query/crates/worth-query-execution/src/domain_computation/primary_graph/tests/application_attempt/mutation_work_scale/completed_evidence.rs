use std::num::NonZeroUsize;

use super::{
    admitted_program, authenticated_principal, idempotency, live_scope, no_demand_mutation_program,
    resolved_account, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_completed_evidence_capacity;

fn world_with_capacity(
    maximum: usize,
) -> crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld {
    installed_authorization_world_with_completed_evidence_capacity(
        NonZeroUsize::new(maximum).expect("a fixture must install finite capacity"),
    )
}

/// Commits `from` -> `to` under `key` and drops the caller's receipt, so only
/// the idempotency window holds the commit's evidence.
fn commit_status(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    from: &str,
    to: &str,
    key: u8,
) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, from, &request);
    let program = admitted_program(world, &principal, &account, &request, to);
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(key, key),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("key {key} commits inside the idempotency window");
    };
    drop(receipt);
}

#[test]
fn a_replay_beyond_the_idempotency_window_is_typed_and_never_re_executes() {
    let probe = world_with_capacity(4 * 1_024 * 1_024);
    let probe_capacity = probe
        .application
        .primary_provider
        .completed_evidence_capacity_for_test();
    let before = probe_capacity.retained_bytes();
    commit_status(&probe, "open", "w1", 91);
    let backing = probe_capacity.retained_bytes() - before;
    drop(probe);

    // The window holds the fixture's own evidence and one more commit's, so
    // each later commit evicts the oldest evidence instead of deferring.
    let world = world_with_capacity(before + backing);
    for (key, from, to) in [(91, "open", "w1"), (93, "w1", "w2"), (95, "w2", "w3")] {
        commit_status(&world, from, to, key);
    }
    let head = world.selected_product().product().selected_commit().clone();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "w3", &request);
    let replay = admitted_program(&world, &principal, &account, &request, "w1");
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            replay,
            idempotency(91, 91),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("a replay beyond the window is denied, never drift or a second execution");
    };
    assert_eq!(
        denial.kind(),
        crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyWindowExpired
    );
    assert_eq!(
        world.selected_product().product().selected_commit(),
        &head,
        "the expired replay did not execute again"
    );
}

#[cfg(feature = "test-query-execution-observer")]
#[test]
fn held_receipts_are_never_evicted_for_room_they_cannot_free() {
    let world = world_with_capacity(4 * 1_024 * 1_024);
    let capacity = world
        .application
        .primary_provider
        .completed_evidence_capacity_for_test();
    let before = capacity.retained_bytes();
    let world = {
        drop(world);
        world_with_capacity(before + 2 * 1_024)
    };
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let mut held = Vec::new();
    let mut state = "open".to_owned();
    for key in (91..).step_by(2).take(64) {
        let account = resolved_account(&world, &state, &request);
        let next = format!("h{key}");
        let program = admitted_program(&world, &principal, &account, &request, &next);
        let (entries, _) = world.application.completed_evidence_retained_for_test();
        match world.application.compare_and_commit_application(
            program,
            idempotency(key, key),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ) {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => held.push(receipt),
            WorthQueryApplicationCommitOutcome::Deferred(_) => {
                let (after, _) = world.application.completed_evidence_retained_for_test();
                assert!(held.len() >= 2, "the window first fills with held receipts");
                assert_eq!(after, entries, "held evidence stays in the window");
                return;
            }
            other => panic!("unexpected outcome {other:?}"),
        }
        state = next;
    }
    panic!("held receipts must eventually exhaust the window");
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
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
