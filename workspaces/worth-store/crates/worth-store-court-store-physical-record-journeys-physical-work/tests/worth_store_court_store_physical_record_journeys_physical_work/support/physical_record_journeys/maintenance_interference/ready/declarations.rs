pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkCapacityDimension,
};

pub(crate) use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture,
};

pub(crate) fn admit_ready_commands(bound: usize) {
    let parent = tempfile::tempdir().unwrap();
    let (profile, request, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(
        bound,
        1,
        1_024,
        1024 * 1024,
        4 * 1024 * 1024,
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(
        &parent.path().join("store"),
        profile.with_capacity(capacity),
    );
    let submission = serving.physical_read_submission();
    for _ in 0..bound {
        match submission.submit(request.clone()).into_raw() {
            worth_proof::TransitionOutcome::Success(_) => {}
            other => panic!("{bound} ready commands must admit, saw {other:?}"),
        }
    }
    let before = serving.media_counters();
    assert!(matches!(
        submission.submit(request).into_raw(),
        worth_proof::TransitionOutcome::Deferred(deferred)
            if deferred.capacity() == bound
                && deferred.dimension() == worth_store::physical_runtime::PhysicalWorkCapacityDimension::Commands
    ));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}
