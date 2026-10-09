//! Performed receipts remain available independently of a synthetic byte window.

use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account, WorthQueryApplicationCommitOutcome,
};

#[test]
fn held_completed_evidence_preserves_exact_old_replay_without_eviction() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let original = admitted_program(&world, &principal, &account, &request, "h91");
    let replay = admitted_program(&world, &principal, &account, &request, "h91");
    let first_outcome = world.application.compare_and_commit_application(
        original,
        idempotency(91, 91),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(first) = first_outcome else {
        panic!("the original prepared intent must commit: {first_outcome:?}");
    };
    let mut held = Vec::new();
    held.push(first);
    let mut state = "h91".to_owned();
    for key in (93..).step_by(2).take(63) {
        let account = resolved_account(&world, &state, &request);
        let next = format!("h{key}");
        let program = admitted_program(&world, &principal, &account, &request, &next);
        let WorthQueryApplicationCommitOutcome::Committed(receipt) =
            world.application.compare_and_commit_application(
                program,
                idempotency(key, key),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        else {
            panic!("each actual status edit commits while prior receipts remain held")
        };
        held.push(receipt);
        state = next;
    }
    let first = &held[0];
    let original_work = first.mutation_work().expect("commit-derived work");
    assert!(!original_work.touched_records().is_empty());
    let head = world.selected_product().product().selected_commit().clone();
    let outcome = world.application.compare_and_commit_application(
        replay,
        idempotency(91, 91),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(replayed) = outcome else {
        panic!("the original prepared retry must recover its retained receipt: {outcome:?}")
    };
    assert_eq!(held.len(), 64);
    assert!(replayed.is_same_authoritative_commit(first));
    assert_eq!(replayed.commit_reference(), first.commit_reference());
    assert_eq!(replayed.mutation_work(), Some(original_work));
    assert_eq!(world.selected_product().product().selected_commit(), &head);
}
