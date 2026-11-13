//! Actual failed managed cleanup at the Directory handoff and cold logical discharge.
use super::*;
#[test]
fn failed_managed_cleanup_records_live_release_and_discharges_after_directory_reopen() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationUncommitted as Uncommitted, WorthQueryHomeRetirementDenial,
    };
    let location = fixture::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let initialized = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("cleanup Home: {cause:?}"));
    let schedule = home
        .runtime()
        .runtime()
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let (_, scope) = pending_support::scope();
    let required = match pending_support::set(home.runtime(), &scope, 0x4847_0701, 29).unwrap() {
        Outcome::Commit(Uncommitted::CleanupRequired(required)) => required,
        other => panic!("the actual native cleanup must refuse: {other:?}"),
    };
    drop(schedule);
    assert!(required.cleanup().durability_denial().is_none());
    let identity = required
        .cleanup()
        .obligation_identity()
        .expect("acknowledged cleanup identity");
    let failure = required.cleanup().failure().kind();
    let run = required.cleanup().failure().run_identity().to_owned();
    let rows = home.runtime().runtime().managed_cleanup_obligations();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identity, identity);
    assert_eq!(rows[0].failure, failure);
    assert_eq!(rows[0].run, run);
    assert!(rows[0].release.is_none());
    let original = rows[0]
        .original
        .expect("actual World original remains associated with failed cleanup");
    let refused = home
        .close(&execution)
        .err()
        .expect("unreleased live resources refuse close");
    assert!(matches!(
        refused.query_denial(),
        Some(WorthQueryHomeRetirementDenial::ManagedCleanup(_))
    ));
    let home = refused
        .resume()
        .unwrap_or_else(|cause| panic!("same original cleanup Home: {cause:?}"));
    let (business, cleanup) = required.into_parts();
    let receipt = home
        .runtime()
        .runtime()
        .with_host_advancement(|phase| cleanup.retry_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|cause| panic!("actual owner-thread release: {cause:?}"));
    assert!(receipt.inspection().resources_released());
    assert_eq!(receipt.inspection().run_identity(), run);
    assert!(home.runtime().runtime().managed_cleanup_obligations()[0]
        .release
        .is_some());
    drop(receipt);
    drop(business);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("recorded resource release close: {cause:?}"))
        .reclaim();
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("initializer replayed")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("fresh cleanup owner: {cause:?}"));
    let rows = home.runtime().runtime().managed_cleanup_obligations();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].identity, identity);
    assert_eq!(rows[0].original, Some(original));
    assert_eq!(rows[0].failure, failure);
    assert_eq!(rows[0].run, run);
    assert!(
        rows[0].release.is_some(),
        "only the recorded actual release admits discharge"
    );
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| {
            home.runtime()
                .runtime()
                .discharge_managed_cleanup(&phase, identity)
        })
        .unwrap()
        .unwrap();
    assert!(home
        .runtime()
        .runtime()
        .managed_cleanup_obligations()
        .is_empty());
    let (_, scope) = pending_support::scope();
    assert!(matches!(
        pending_support::set(home.runtime(), &scope, 0x4847_0701, 29).unwrap(),
        Outcome::RecoveredCommitted(_)
    ));
    assert_eq!(pending_support::balance(home.runtime(), &scope), 29);
    assert_eq!(
        contacts.load(Ordering::SeqCst),
        1,
        "cleanup never reexecutes the business effect"
    );
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("discharged cleanup close: {cause:?}"))
        .reclaim();
}
