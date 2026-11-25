//! An original medium refusal never acknowledges or installs a successor.
use super::{
    candidate, main_basis, published_snapshots, recovered_world, Denial,
    RelationalPublicationOutcome, TransitionError,
};
use worth_relational::facade::{
    durability::RelationalDurabilityUnresolvedSubject, mvcc::RelationalPublicationDenial,
};
use worth_testkit_relational::{
    durable_medium::ScriptedPublication,
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn refused_durable_transition_returns_its_authority_and_moves_nothing() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority, medium, _members) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    medium.script_next_publication(ScriptedPublication::ProvenNoEffect);
    let prepared = candidate(execution, &runtime, "refused-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("a refused durable publication is a pre-effect transition refusal")
    };
    assert!(matches!(refusal.denial(), Denial::Publication(outcome)
    if matches!(outcome.as_ref(), RelationalPublicationOutcome::Denied(
        RelationalPublicationDenial::DurablePublicationRefused(_)
    ))));
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(published_snapshots(execution, &runtime).0, 0);
    assert_eq!(
        medium
            .committed_artifacts()
            .unwrap()
            .last()
            .unwrap()
            .position()
            .0,
        1
    );

    let authority = refusal.into_authority();
    let retried = candidate(execution, &runtime, "retried-transition");
    let transition = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, retried)
        .expect("the refused position was never consumed");
    assert_eq!(
        medium
            .committed_artifacts()
            .unwrap()
            .last()
            .unwrap()
            .position()
            .0,
        2
    );
    assert_eq!(runtime.history().immutable_commit_count(), count + 1);
    assert!(transition
        .into_parts()
        .1
        .admit_basis(main_basis(&runtime))
        .is_ok());
}

#[test]
fn unresolved_durable_transition_installs_nothing_and_names_its_fate() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut runtime, authority, medium, _members) = recovered_world(execution);
    let predecessor = main_basis(&runtime);
    let count = runtime.history().immutable_commit_count();
    medium.script_next_publication(ScriptedPublication::IndeterminateWithoutEffect);
    let prepared = candidate(execution, &runtime, "unresolved-transition");
    let TransitionError::Refused(refusal) = runtime
        .durability_recovery()
        .commit_checkpoint_transition(execution, authority, prepared)
        .unwrap_err()
    else {
        panic!("an unresolved durable publication installs no successor")
    };
    let Denial::Publication(outcome) = refusal.denial() else {
        panic!("the refusal names the publication outcome")
    };
    let RelationalPublicationOutcome::DurabilityUnresolved(fate) = outcome.as_ref() else {
        panic!("the medium could not say whether the transition is durable")
    };
    let offered = medium.offered().pop().expect("the medium was asked first");
    let RelationalDurabilityUnresolvedSubject::CanonicalCommit {
        position, digest, ..
    } = fate.subject()
    else {
        panic!("a commit's unresolved fate names the commit")
    };
    assert_eq!(position, offered.position());
    assert_eq!(digest, offered.payload_digest());
    assert_eq!(main_basis(&runtime).descriptor(), predecessor.descriptor());
    assert_eq!(runtime.history().immutable_commit_count(), count);
    assert_eq!(published_snapshots(execution, &runtime).0, 0);
}
