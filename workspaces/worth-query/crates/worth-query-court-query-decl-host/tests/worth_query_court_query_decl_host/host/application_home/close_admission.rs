//! A real installed call fences World close before any service is sealed.
use super::*;
use std::sync::mpsc;

#[test]
fn active_installed_call_refuses_before_world_seal_and_resumes_the_same_home() {
    let location = fixture::location();
    let request = fixture::request(
        location,
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    );
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("genuine directory opening: {refused:?}"));
    // This clone escapes the outer program-runtime Arc. Its Native/Bridge
    // shares must not turn a later refusal into the World-admission oracle.
    let product = home.runtime().product_runtime().clone();
    let (entered, wait_entered) = mpsc::sync_channel(0);
    let (release, wait_release) = mpsc::sync_channel(0);
    let worker = std::thread::spawn(move || {
        product.with_advancement(|_| {
            entered.send(()).unwrap();
            // Sender abandonment also releases the worker after a test failure.
            let _ = wait_release.recv();
        })
    });
    wait_entered
        .recv_timeout(Duration::from_secs(60))
        .expect("the actual installed advancement entered its body");
    let refused = home
        .close(&execution)
        .err()
        .expect("an actual admitted call prevents World sealing");
    // Host exposes this owner's typed denial through its existing diagnostic.
    // Native sharing or Bridge retention has a different actual cause and
    // cannot restore this pre-seal World posture.
    assert_eq!(refused.detail(), "World(Denied(InFlightCriticalSection))");
    let home = refused
        .resume()
        .unwrap_or_else(|refused| panic!("same Home pre-seal resume: {refused:?}"));
    assert!(home.is_directory());
    release.send(()).unwrap();
    worker.join().expect("the admitted worker exits").unwrap();
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(),
        );
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let observed = runtime
            .request(&principal, &scope)
            .query(fixture::balance_query::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .expect("the resumed actual Home still serves its original value");
        assert_eq!(observed.rows(), &[7]);
    }
    let closed = home
        .close(&execution)
        .unwrap_or_else(|refused| panic!("owning close after call release: {refused:?}"));
    assert!(closed.is_closed());
}
