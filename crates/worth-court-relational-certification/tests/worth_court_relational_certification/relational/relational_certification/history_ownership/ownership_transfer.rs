//! D1188 selects a transferred member's original identity and outside owner.
use super::transfer_fixture::*;
use super::*;
#[test]
fn a_transfer_round_trip_restores_the_selected_single_owner_without_touching_the_other_document() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    round_trip(execution, execution);
}
#[test]
fn a_worker_only_transfer_round_trip_uses_the_original_request_without_accounting() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let worker = &ExecutionRequest::leased(&lease);
    assert!(!worker.has_accounting());
    round_trip(setup, worker);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
fn round_trip(setup: &ExecutionRequest<'_, '_>, navigation: &ExecutionRequest<'_, '_>) {
    let runtime = installed_runtime();
    let a = create_entity(setup, &runtime, "a");
    let b = create_entity(setup, &runtime, "b");
    let e = member(setup, &runtime, "member");
    let untouched = member(setup, &runtime, "other-b-member");
    create_relation(setup, &runtime, b, untouched, "b-other-owner");
    let edge = create_relation(setup, &runtime, a, e, "owner");
    let before = basis(&runtime);
    move_to(setup, &runtime, edge, b, e);
    let transferred = basis(&runtime);
    let mut read = begin(&runtime);
    let b_before = read.read_entity(setup, b).unwrap();
    let other_before = read.read_entity(setup, untouched).unwrap();
    for (selected, owner) in [(&before, a), (&transferred, b), (&before, a)] {
        let committed = select(navigation, &runtime, a, selected).unwrap();
        assert_owner(setup, &runtime, e, edge, owner, &[a, b]);
        let mut read = begin(&runtime);
        assert_eq!(read.read_entity(setup, b).unwrap(), b_before);
        assert_eq!(read.read_entity(setup, untouched).unwrap(), other_before);
        assert!(committed.changed_records.contains(&RecordRef::Entity(e)));
        assert!(committed
            .changed_records
            .contains(&RecordRef::Relation(edge)));
        assert!(!committed.changed_records.contains(&RecordRef::Entity(b)));
        release_test_commit_snapshot(&runtime, &committed);
    }
}
#[test]
fn three_document_transfer_navigation_follows_each_selected_owner() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let c = create_entity(execution, &runtime, "c");
    let e = member(execution, &runtime, "member");
    let edge = create_relation(execution, &runtime, a, e, "owner");
    let at_a = basis(&runtime);
    move_to(execution, &runtime, edge, b, e);
    let at_b = basis(&runtime);
    move_to(execution, &runtime, edge, c, e);
    let at_c = basis(&runtime);
    for (selected, owner) in [(&at_a, a), (&at_b, b), (&at_c, c), (&at_b, b), (&at_a, a)] {
        let committed = select(execution, &runtime, a, selected).unwrap();
        assert_owner(execution, &runtime, e, edge, owner, &[a, b, c]);
        release_test_commit_snapshot(&runtime, &committed);
    }
}
