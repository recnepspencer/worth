//! Actual installed host/result admission: small reads must not retain the
//! broad host envelope, and the cap must refuse actual oversized projections.
use super::*;

fn narrow_limits() -> WorthQueryApplicationQueryBatchLimits {
    WorthQueryApplicationQueryBatchLimits::new(
        NonZeroUsize::new(8).unwrap(),
        NonZeroUsize::new(1024).unwrap(),
        NonZeroUsize::new(128 * 1024).unwrap(),
        NonZeroUsize::new(8 * 1024).unwrap(),
    )
}

#[test]
fn broad_host_profile_is_narrowed_before_real_batch_planning_and_custody() {
    let world = CourtroomWorld::publish_with_result_bytes("ready", 320 * 1024 * 1024);
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let observer = world.application.result_buffer_observer();
    let before = observer.observe().retained_bytes();
    let batch = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1"), read("intent-1")])
        .limits(narrow_limits())
        .execute()
        .unwrap();
    assert_eq!(batch.items().len(), 2);
    assert_eq!(batch.items()[0].rows()[0].input, "payload");
    assert!(batch.receipt().read_work().peak_bytes() <= 128 * 1024);
    assert!(batch.items().iter().all(|item| item
        .receipt()
        .inspect()
        .read_work()
        .total_work_units()
        > 0));
    drop(batch);
    assert_eq!(observer.observe().retained_bytes(), before);
    assert_eq!(observer.observe().active_buffers(), 0);
}

#[test]
fn oversized_actual_field_refuses_the_narrow_buffer_without_collection_or_leak() {
    let mut world = CourtroomWorld::publish_with_result_bytes("ready", 320 * 1024 * 1024);
    world.supersede_intent(2, 5, "active", &"x".repeat(16 * 1024), "ready");
    let scope = world::request_scope();
    let auth = world::admit_identity_adapter(world.application.installed_schema());
    let external = block_on(auth.authenticate((), &scope)).unwrap();
    let observation = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    let observer = world.application.result_buffer_observer();
    let before = observer.observe().retained_bytes();
    let outcome = world
        .application
        .request(&external, &scope)
        .at(&observation)
        .query_batch(vec![read("intent-1")])
        .limits(narrow_limits())
        .execute();
    assert!(
        matches!(outcome, Err(WorthQueryApplicationQueryBatchDenial::Item {
        index: 0,
        cause: worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Execution(error)
    }) if error.kind() == worth_query_host::facade::primary_graph::WorthQueryApplicationOneShotDenialKind::ResultBufferLimitExceeded)
    );
    assert_eq!(observer.observe().retained_bytes(), before);
    assert_eq!(observer.observe().active_buffers(), 0);
    let current = world
        .application
        .request(&external, &scope)
        .retain_read()
        .unwrap();
    assert_eq!(current.selected_commit(), observation.selected_commit());
}
