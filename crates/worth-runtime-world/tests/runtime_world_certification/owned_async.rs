#[path = "owned_async/fixture.rs"]
mod fixture;

#[path = "owned_async/managed.rs"]
mod managed;

use fixture::{completion, source_commit, AsyncSourceCourt};
use worth_runtime_bridge::facade::TruthCommitIdentity;
use worth_runtime_world::facade::{
    RuntimeWorldOwnedAsyncRequestAdmissionDenial, RuntimeWorldOwnedAsyncRevalidationDenial,
};

#[test]
fn populated_zero_version_fork_keeps_native_commit_through_completion_and_revalidation() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = AsyncSourceCourt::new();
    let committed = court.publish(execution, &court.empty);
    let fork = court.fork(&committed, "async-fork");
    let native = fork.basis().relational_basis().observation();
    assert_eq!(
        native.truth_version().as_u64(),
        0,
        "fork resets the branch counter"
    );
    let selected = native
        .commit_id()
        .expect("fork retains a committed native target");
    assert_ne!(selected.0, native.truth_version().as_u64());

    // Sibling publication and abandoned preparation move global issuance without
    // moving the selected fork. Neither is authority for its source commit.
    drop(court.prepare(&execution, &committed));
    let sibling = court.publish(execution, &committed);
    let sibling_commit = sibling
        .basis()
        .relational_basis()
        .observation()
        .commit_id()
        .unwrap();
    assert!(
        sibling_commit.0 > selected.0 + 1,
        "abandoned preparation burns an occurrence"
    );
    let lease = court.lease(&fork);
    let request = court
        .world
        .admit_owned_async_request(&court.bridge, &court.declaration, &fork, &lease)
        .expect("populated fork reaches the real Signal admission");
    let expected = TruthCommitIdentity::from_relational_commit_id(selected.0);
    assert_eq!(
        source_commit(&request),
        &expected,
        "admitted source equals native reference target"
    );
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&request)
            .unwrap(),
        1
    );
    let validated = court
        .bridge
        .validate_owned_async_completion_envelope(&request, completion(&request))
        .unwrap();
    let completed = court
        .bridge
        .admit_owned_async_completion(&request, &validated)
        .unwrap();
    assert!(completed.report().admitted_completion().is_some());
    assert_eq!(source_commit(&request), &expected);
    court.bridge.retire_owned_async_request(&request).unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&request)
            .unwrap(),
        0
    );

    let request = court
        .world
        .admit_owned_async_request(&court.bridge, &court.declaration, &fork, &lease)
        .unwrap();
    let sibling_lease = court.lease(&sibling);
    let replacement = court
        .world
        .revalidate_owned_async_request(&court.bridge, &request, &sibling, &sibling_lease)
        .unwrap();
    assert_eq!(
        source_commit(replacement.request()),
        &TruthCommitIdentity::from_relational_commit_id(sibling_commit.0)
    );
    assert_eq!(
        source_commit(&request),
        &expected,
        "original binding remains immutable"
    );
    let obsolete = court
        .bridge
        .validate_owned_async_completion_envelope(&request, completion(&request))
        .unwrap();
    let obsolete = court
        .bridge
        .admit_owned_async_completion(&request, &obsolete)
        .unwrap();
    assert!(obsolete.report().admitted_completion().is_none());
    assert!(
        obsolete.report().denied_completion().is_some(),
        "superseded request cannot complete its replacement"
    );
    court
        .bridge
        .retire_owned_async_request(replacement.request())
        .unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(replacement.request())
            .unwrap(),
        0
    );
}

#[test]
fn empty_source_refuses_before_signal_and_preserves_original_request_custody() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let court = AsyncSourceCourt::new();
    let empty_lease = court.lease(&court.empty);
    let denial = court.world.admit_owned_async_request(
        &court.bridge,
        &court.declaration,
        &court.empty,
        &empty_lease,
    );
    assert!(matches!(
        denial,
        Err(RuntimeWorldOwnedAsyncRequestAdmissionDenial::NoCommittedSource)
    ));

    let committed = court.publish(execution, &court.empty);
    let committed_lease = court.lease(&committed);
    let request = court
        .world
        .admit_owned_async_request(
            &court.bridge,
            &court.declaration,
            &committed,
            &committed_lease,
        )
        .expect("committed twin admits with unchanged installed declaration");
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&request)
            .unwrap(),
        1,
        "empty refusal created no hidden request"
    );
    let denied = court.world.revalidate_owned_async_request(
        &court.bridge,
        &request,
        &court.empty,
        &empty_lease,
    );
    assert!(matches!(
        denied,
        Err(RuntimeWorldOwnedAsyncRevalidationDenial::NoCommittedSource)
    ));
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&request)
            .unwrap(),
        1,
        "refused revalidation preserves original custody"
    );

    let mismatch = court.world.admit_owned_async_request(
        &court.bridge,
        &court.declaration,
        &committed,
        &empty_lease,
    );
    assert!(matches!(
        mismatch,
        Err(RuntimeWorldOwnedAsyncRequestAdmissionDenial::RelationalSourceMismatch)
    ));
    let foreign = AsyncSourceCourt::new();
    let foreign_denial = foreign.world.admit_owned_async_request(
        &court.bridge,
        &court.declaration,
        &committed,
        &committed_lease,
    );
    assert!(matches!(
        foreign_denial,
        Err(RuntimeWorldOwnedAsyncRequestAdmissionDenial::ForeignOwner)
    ));
    court.bridge.retire_owned_async_request(&request).unwrap();
    assert_eq!(
        court
            .bridge
            .owned_async_active_request_count(&request)
            .unwrap(),
        0
    );
}
