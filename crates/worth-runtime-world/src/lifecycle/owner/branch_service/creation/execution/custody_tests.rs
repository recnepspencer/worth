use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use super::CreationDestination;
use crate::branch::OwnerCreatedComponentCustodyRecord;
use crate::lifecycle::owner::branch_service::tests::fork_creation::{
    fork_intent, relational_fork, setup_with_relational_source, signal_fork,
};
use crate::lifecycle::RuntimeWorldPreparationService;
use crate::publication::{
    CompositeAttemptProgress, RuntimeWorldCancellationSource, SignalAttemptProgress,
    SignalAttemptProgressPosture,
};
use crate::recovery::{ProductUnpublishedCause, ProductUnpublishedRetentionPosture};

#[test]
fn unwind_after_first_actual_creation_fork_retains_exact_effect_without_signal_contact() {
    let (fixture, owner, source) = setup_with_relational_source(3);
    let intent = fork_intent(
        "first-fork-unwind",
        relational_fork("rel-first-fork-unwind"),
        signal_fork("signal-never-forked"),
    );
    let mut reservation = owner
        .state
        .branches
        .reserve_branch(owner.owner_identity(), intent.name().clone())
        .unwrap();
    let (branch, incarnation) = owner
        .issue_branch_identities(intent.name().clone())
        .unwrap();
    let witness = reservation
        .bind_creation_destination(branch.clone(), incarnation)
        .unwrap();
    let destination = CreationDestination {
        witness,
        branch,
        incarnation,
    };
    let cancellation = RuntimeWorldCancellationSource::new();
    let mut attempt = owner
        .prepare_creation(source.clone(), intent, &cancellation.token(), None)
        .unwrap();
    attempt.bind_destination(Arc::clone(&destination.witness));
    attempt.begin_owner_execution();
    let returned = super::fork_relational(&owner, &mut attempt, &destination)
        .unwrap_or_else(|_| panic!("actual Relational fork succeeds"));
    let (_, results) = CompositeAttemptProgress::new(returned, SignalAttemptProgress::untouched())
        .ready_results()
        .unwrap();
    let fork_target = results.relational_fork_target_identity().unwrap().clone();
    assert_eq!(attempt.counters().relational_owner_contacts(), 1);
    assert_eq!(attempt.counters().signal_owner_contacts(), 0);
    let owed = owner
        .state
        .custody
        .installed_records()
        .into_iter()
        .map(OwnerCreatedComponentCustodyRecord::into_retirement_work)
        .collect::<Vec<_>>();
    assert_eq!(owed.len(), 1);
    assert!(catch_unwind(AssertUnwindSafe(move || {
        let _attempt = attempt;
        let _destination = reservation;
        panic!("caller unwinds after the first real fork returns");
    }))
    .is_err());
    let handle = owner.recovery_handles().pop().unwrap();
    let record = owner.inspect_recovery(&handle).unwrap();
    assert_eq!(record.cause(), ProductUnpublishedCause::CallerAbandoned);
    assert_eq!(
        record.destination_branch(),
        Some((&destination.branch, destination.incarnation))
    );
    assert_eq!(
        record.component_results().relational_fork_target_identity(),
        Some(&fork_target)
    );
    assert_eq!(record.owner_effect_count(), 1);
    assert_eq!(
        record.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert_eq!(
        record.retention_posture(),
        ProductUnpublishedRetentionPosture::BindingReserved
    );
    assert!(record.successor_commit().is_none());
    assert_eq!(
        owner.state.branches.root_cell().unwrap().atomic_snapshot(),
        *source.snapshot()
    );
    assert!(
        fixture
            .reserve_relational_fork_target("rel-first-fork-unwind")
            .is_err(),
        "the returned fork exists in its actual owner"
    );
    drop(record);
    assert_eq!(owner.cleanup_recovery_handle(&handle).unwrap(), owed);
    assert_eq!(owner.state.custody.installed(), 0);
    assert_eq!(owner.state.retention.reserved_unique_pin_capacity(), 0);
    assert_eq!(owner.state.history.reserved_len(), 0);
    assert_eq!(owner.state.operation.active(), 0);
}
