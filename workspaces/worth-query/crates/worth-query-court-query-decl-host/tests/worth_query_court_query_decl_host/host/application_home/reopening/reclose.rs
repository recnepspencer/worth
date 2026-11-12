//! A Home closed again before anything reads its indexes keeps every carried selection.
use super::super::pending_support::{balance, scope, set};
use super::super::*;
use super::opening_failed;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

#[test]
fn reclose_before_any_read_keeps_carried_index_selections_for_the_next_reopen() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let open = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open.open_in_advancement(&phase)
                .unwrap_or_else(|refused| panic!("fresh Home: {refused:?}")),
        )
    })
    .unwrap();
    let (_, live) = scope();
    // One write after the identity index was built: that selection is carried.
    assert!(matches!(
        set(home.runtime(), &live, 0x781, 8).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
    ));
    (*home)
        .close(&execution)
        .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
    let reopen = |stage: &'static str| {
        let open = fixture::request_with_initializer(
            location.clone(),
            build,
            contacts.clone(),
            |_, _, _| panic!("existing Home initializer must never run"),
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(
                open.open_in_advancement(&phase)
                    .unwrap_or_else(|refused| opening_failed(stage, &refused)),
            )
        })
        .unwrap()
    };
    // Reopened and closed with no request. That this close captures the
    // selections from the never-demanded image is observed by the Native court
    // `complete_index_selections::reclose_before_any_image_demand_keeps_the_
    // selections_the_image_carried`; here the next reopen must still read them.
    let untouched = reopen("untouched readmission");
    (*untouched)
        .close(&execution)
        .unwrap_or_else(|refused| panic!("re-close before any read: {refused:?}"));
    let home = reopen("readmission after re-close");
    assert_eq!(balance(home.runtime(), &live), 8);
    assert!(matches!(
        set(home.runtime(), &live, 0x782, 9).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { result: 9, .. }
    ));
    assert_eq!(balance(home.runtime(), &live), 9);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    (*home)
        .close(&execution)
        .unwrap_or_else(|refused| panic!("reader close: {refused:?}"));
}
