//! A real Native preparation stop leaves an empty World original for cold release.
use super::*;
use worth_query_host::facade::primary_graph::{
    CommitExecutionDenialKind, RelationalExecutionDenialCause, TransactionCommitError,
    WorldNativePreparationDenial,
};

#[test]
fn an_empty_completion_original_is_released_on_reopen_before_the_callback_retries() {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "empty-original-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8451, &scope);
    let body = route.message(&owners, &effect, 81);
    let admitted = home
        .runtime()
        .runtime()
        .authenticate_inbound_occurrence(&route.handle, &body)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();
    let (cancel, stopped) = pending_support::scope();
    let pause = home
        .runtime()
        .runtime()
        .pause_after_source_publication_validation_for_test(NonZeroUsize::MIN);
    let receipt = std::thread::scope(|threads| {
        let controller = threads.spawn(move || {
            assert!(
                pause.wait_until_reached(Duration::from_secs(20)),
                "actual durable open precedes Native completion preparation"
            );
            cancel.cancel();
            pause.release();
        });
        let receipt = home
            .runtime()
            .runtime()
            .with_application_advancement(&stopped, |phase| {
                admitted.execute_in_advancement(&phase, &stopped)
            })
            .unwrap()
            .unwrap();
        controller.join().unwrap();
        receipt
    });
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    let Some(WorldNativePreparationDenial::Native(TransactionCommitError::Execution {
        denial,
        ..
    })) = receipt.preparation_denial()
    else {
        panic!(
            "the actual Native preparation must observe cancellation: {:?}",
            receipt.preparation_denial()
        );
    };
    assert_eq!(
        denial.kind,
        CommitExecutionDenialKind::Cause(RelationalExecutionDenialCause::Cancelled)
    );
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    assert_eq!(rail.contacts().len(), 1);
    drop(receipt);
    end_live_home_for_required_reopen(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "empty-original-audience");
    assert_eq!(route.message(&owners, &effect, 81), body);
    assert_eq!(
        maintain(&home, &route).performed(),
        1,
        "cold release leaves the accepted callback retryable"
    );
    let original = terminal(&home, &route, &effect);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::Performed));
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);

    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "empty-original-audience");
    assert_eq!(route.message(&owners, &effect, 81), body);
    assert_eq!(terminal(&home, &route, &effect), original);
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    assert_eq!(owners.handlers.load(Ordering::SeqCst), 1);
    close(home, &execution);
}
