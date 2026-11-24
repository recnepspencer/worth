//! Refuse the original live retry until the recorded opening can be readmitted.
use super::*;

#[test]
fn killed_creation_without_owner_forks_reopens_the_same_custody_and_only_releases() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    // The intent lands, its writer cannot tell, and no owner runs.
    f.recorder.undecide_next(true);
    let killed = kill(&f, &request, &parent, "unforked", true, true);
    assert_eq!(frontiers(&f, &request), before, "no owner fork ran");
    let original = f
        .world
        .recovery_port()
        .inspect_effects(&killed.stale)
        .unwrap();
    assert!(matches!(
        original.cause(),
        ProductUnpublishedCause::WorldOccurrenceReadmissionRequired
    ));
    assert_eq!(
        original.next_actions(),
        &[
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::ReopenRequired,
        ]
    );
    drop(original);
    for _ in 0..2 {
        assert!(matches!(
            resume(&f, &request, &killed.stale),
            Err(RuntimeWorldRecoveryDenial::ReopenRequired)
        ));
    }
    assert_eq!(frontiers(&f, &request), before, "retry runs no owner");
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "unforked");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::ReleasedOriginal(receipt) = outcome else {
        panic!("an effect-free original only releases: {outcome:?}")
    };
    assert_eq!(receipt.association().attempt().ordinal(), killed.attempt);
    assert_eq!(frontiers(&f, &request), before, "release runs no owner");
    assert!(matches!(
        resume(&f, &request, &row),
        Err(RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    // The durable release lifts the fence; a fresh creation issues new identities.
    let parent = f
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(fresh) =
        create(&f, &request, &parent, "unforked", false, false)
    else {
        panic!("the released name is free")
    };
    assert!(fresh.lifecycle_incarnation().ordinal() > killed.incarnation);
    drop((receipt, parent, fresh, row));
    let f = f.reopen(&request);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    assert_eq!(descriptions(&f, &request).len(), 2);
    f.finish(&request);
}

#[test]
fn refused_selection_disposition_after_owner_forks_requires_reopen_and_continues_without_reforking()
{
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    // Intent lands (#1), both owners fork, then final BranchCreated write (#2) is refused.
    f.recorder.crash_after(1);
    let killed = kill(&f, &request, &parent, "refused-selection", true, true);
    let forked = frontiers(&f, &request);
    assert_ne!(forked.0, before.0, "Native forked before refusal");
    assert_ne!(forked.1, before.1, "Signal forked before refusal");
    let original = f
        .world
        .recovery_port()
        .inspect_effects(&killed.stale)
        .unwrap();
    assert!(matches!(
        original.cause(),
        ProductUnpublishedCause::WorldOccurrenceReadmissionRequired
    ));
    assert_eq!(
        original.next_actions(),
        &[
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::ReopenRequired,
        ]
    );
    drop(original);
    for _ in 0..2 {
        assert!(matches!(
            resume(&f, &request, &killed.stale),
            Err(RuntimeWorldRecoveryDenial::ReopenRequired)
        ));
    }
    assert_eq!(frontiers(&f, &request), forked, "retry runs no owner");
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "refused-selection");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("a performed original continues after reopen: {outcome:?}")
    };
    assert_eq!(child.branch_identity().name().as_str(), "refused-selection");
    assert_eq!(child.lifecycle_incarnation().ordinal(), killed.incarnation);
    assert_eq!(frontiers(&f, &request), forked, "no owner fork repeats");
    let selected = descriptions(&f, &request);
    assert_eq!(selected.len(), 2);
    drop((child, row));
    let f = f.reopen(&request);
    assert_eq!(descriptions(&f, &request), selected);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    f.finish(&request);
}

#[test]
fn refused_unsubmitted_release_disposition_requires_reopen_and_only_releases() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let before = frontiers(&f, &request);
    // Hold intent first so cancellation enters after intent ACK with zero owner work.
    f.recorder.hold_publication(true);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(pending) =
        create(&f, &request, &parent, "refused-release", true, true)
    else {
        panic!("held intent remains unpublished")
    };
    let (_, incarnation) = pending.destination_branch().expect("reserved destination");
    let killed = Killed {
        stale: pending.recovery_handle(),
        attempt: pending.attempt_identity().ordinal(),
        custody: pending.identity().ordinal(),
        incarnation: incarnation.ordinal(),
    };
    drop(pending);
    let cancellation = RuntimeWorldCancellationSource::new();
    cancellation.cancel();
    f.recorder.hold_publication(false);
    // Intent ACK commits (#1), then unsubmitted CreationReleased (#2) is refused.
    f.recorder.crash_after(1);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) = f
        .world
        .branch_port()
        .resume_creation(&request, &killed.stale, &cancellation.token())
        .unwrap()
    else {
        panic!("refused release disposition retains the original for reopen")
    };
    assert!(matches!(
        effects.cause(),
        ProductUnpublishedCause::WorldOccurrenceReadmissionRequired
    ));
    assert_eq!(
        effects.next_actions(),
        &[
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::ReopenRequired,
        ]
    );
    drop(effects);
    for _ in 0..2 {
        assert!(matches!(
            resume(&f, &request, &killed.stale),
            Err(RuntimeWorldRecoveryDenial::ReopenRequired)
        ));
    }
    assert_eq!(frontiers(&f, &request), before, "no owner fork ran");
    drop(parent);
    let (f, row) = reopen_undecided(f, &request, &killed, "refused-release");
    let outcome = resume(&f, &request, &row).unwrap();
    let RuntimeWorldBranchCreationOutcome::ReleasedOriginal(receipt) = outcome else {
        panic!("unsubmitted original releases after reopen: {outcome:?}")
    };
    assert_eq!(receipt.association().attempt().ordinal(), killed.attempt);
    assert_eq!(frontiers(&f, &request), before, "release runs no owner");
    assert_eq!(descriptions(&f, &request).len(), 1);
    drop((receipt, row));
    let f = f.reopen(&request);
    assert_eq!(
        f.world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    assert_eq!(descriptions(&f, &request).len(), 1);
    f.finish(&request);
}
