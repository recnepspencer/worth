//! The same physical Home, with no initializer or business-handler replay.
use super::pending_support::{balance, scope, set};
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
mod checkpoint_restart;
mod refusals;
mod separate_process;

pub(crate) fn opening_failed(stage: &str, refused: &impl std::fmt::Debug) -> ! {
    let detail = format!("{refused:?}");
    let end = detail
        .char_indices()
        .nth(1800)
        .map_or(detail.len(), |(index, _)| index);
    panic!("{stage}: {}", &detail[..end]);
}

#[test]
fn same_directory_reopens_reads_and_commits_without_replaying_initializer_or_old_key() {
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
        open.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("fresh Home: {refused:?}"));
    let (_, live) = scope();
    let original_commit = match set(home.runtime(), &live, 0x771, 8).unwrap() {
        WorthQueryApplicationMutationOutcome::Committed { receipt, result: 8 } => {
            receipt.commit_reference().commit_id
        }
        other => panic!("writer must commit balance eight: {other:?}"),
    };
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
    let open = fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
        panic!("existing Home initializer must never run")
    });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        open.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("same Home readmission", &refused));
    assert_eq!(balance(home.runtime(), &live), 8);
    match set(home.runtime(), &live, 0x771, 8).unwrap() {
        WorthQueryApplicationMutationOutcome::RecoveredCommitted(observed) => {
            assert_eq!(observed.commit_reference().commit_id, original_commit);
        }
        other => panic!("old key must return its exact original durable receipt: {other:?}"),
    }
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    assert!(matches!(
        set(home.runtime(), &live, 0x772, 9).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { result: 9, .. }
    ));
    assert_eq!(balance(home.runtime(), &live), 9);
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("reader close: {refused:?}"));
}

#[test]
fn flushed_and_pending_multibatch_homes_readmit_their_completed_publication() {
    for flush_final in [true, false] {
        let location = fixture::location();
        let contacts = Arc::new(AtomicUsize::new(0));
        let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        let request = fixture::request_with_initializer(
            location.clone(),
            build,
            contacts.clone(),
            move |phase, graph, schema| {
                fixture::schema::seed_batches(phase, graph, schema, flush_final)
            },
        );
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| panic!("multibatch bootstrap: {refused:?}"));
        home.close(&execution)
            .unwrap_or_else(|refused| panic!("writer close: {refused:?}"));
        let request =
            fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
                panic!("multibatch reopening must not initialize")
            });
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .unwrap()
        .unwrap_or_else(|refused| opening_failed("multibatch readmission", &refused));
        let (_, live) = scope();
        assert_eq!(balance(home.runtime(), &live), 7);
        assert!(matches!(
            set(home.runtime(), &live, 0x773, 8).unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { result: 8, .. }
        ));
        assert_eq!(balance(home.runtime(), &live), 8);
        assert_eq!(contacts.load(Ordering::SeqCst), 1);
        home.close(&execution)
            .unwrap_or_else(|refused| panic!("reader close: {refused:?}"));
    }
}
