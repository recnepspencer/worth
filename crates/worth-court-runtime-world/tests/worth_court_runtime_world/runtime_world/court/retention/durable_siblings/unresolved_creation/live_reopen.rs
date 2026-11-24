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
