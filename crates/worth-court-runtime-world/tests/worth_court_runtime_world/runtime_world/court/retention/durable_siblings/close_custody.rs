//! A durable Home closes with published siblings; unpublished sibling custody refuses.
use super::*;

#[test]
fn durable_home_closes_with_published_mixed_fork_siblings() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    let first = match create(&f, &request, &parent, "published", true, true) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("published actual sibling: {other:?}"),
    };
    let nested = match create(&f, &request, &first, "nested", true, true) {
        RuntimeWorldBranchCreationOutcome::Performed(actual) => actual,
        other => panic!("nested actual sibling: {other:?}"),
    };
    assert_eq!(descriptions(&f, &request).len(), 3);
    drop((parent, first, nested));
    // Both siblings are installed after their durable selection; their creation
    // records stay with the readmitted artifact, so graceful close succeeds.
    f.finish(&request);
}

#[test]
fn durable_home_close_refuses_an_unpublished_sibling_until_its_original_completes() {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut f = Fixture::journal(&request);
    let parent = root(&mut f, &request);
    f.recorder.hold_publication(true);
    let RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) =
        create(&f, &request, &parent, "unpublished", true, true)
    else {
        panic!("the actual medium holds the original before its durable selection")
    };
    let handle = effects.recovery_handle();
    drop(effects);
    assert_eq!(
        f.world
            .lifecycle_port()
            .close(&request, &f.world)
            .unwrap_err(),
        RuntimeWorldServiceDenial::Denied(RuntimeWorldCloseDenial::UnresolvedBusinessCustody),
        "an unpublished sibling original keeps its custody unresolved"
    );
    f.recorder.hold_publication(false);
    let outcome = f
        .world
        .branch_port()
        .resume_creation(
            &request,
            &handle,
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let RuntimeWorldBranchCreationOutcome::Performed(child) = outcome else {
        panic!("the same original completes: {outcome:?}")
    };
    drop((child, parent));
    f.finish(&request);
}
