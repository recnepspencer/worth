//! Actual interrupted matching cannot return its already-started complete selection.
use super::*;
use std::sync::atomic::Ordering;

#[test]
fn complete_membership_visits_nonmatches_and_refuses_interrupted_partial_selection() {
    run(journey);
}
fn journey() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone());
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 7);
    let index = request
        .query(find(""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("")).rebuild_index(&index).unwrap();
    let reached = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = reached.clone();
    *calls.after_second_match.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancellation.cancel();
    }));
    let error = request
        .query(find(""))
        .execute()
        .err()
        .expect("partial traversal refuses");
    assert!(
        reached.load(Ordering::SeqCst),
        "two genuine Source predicates ran"
    );
    assert_eq!(calls.matched_sources.load(Ordering::SeqCst), 2);
    assert!(
        matches!(error,
        WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause)
        if matches!(cause.cause(), WorthQueryManagedIndexDenialCause::Work(
            WorthQueryManagedIndexWorkDenial::Controlled(WorthQueryManagedIndexStop::Request(
                WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
            ))))),
        "{error:?}"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    for probe in ["missing", "", "alpha", "ál"] {
        let before = calls.matched_sources.load(Ordering::SeqCst);
        let actual = fresh.query(find(probe)).execute().unwrap();
        assert_eq!(actual.rows(), journey::expected(source.rows(), probe));
        assert_eq!(
            calls.matched_sources.load(Ordering::SeqCst) - before,
            7,
            "explicit exhaustive membership visits every Source, including nonmatches"
        );
    }
    assert_eq!(calls.builds.load(Ordering::SeqCst), 1);
    index.close();
}
