//! A real closed directory names the incompatible request before owner effects.
use super::*;
use std::sync::atomic::AtomicBool;

#[test]
fn changed_program_requires_a_transition_before_initialization_or_supplier_execution() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    initialize_predecessor(
        execution,
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    refuse_successor(execution, location.clone(), build, contacts.clone());
    reopen_predecessor(execution, location, build, contacts.clone());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[inline(never)]
fn initialize_predecessor(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    initialized: Arc<AtomicUsize>,
    contacts: Arc<AtomicUsize>,
) {
    let request = fixture::request(
        location.clone(),
        build,
        initialized.clone(),
        contacts.clone(),
    );
    let mut opened = None;
    with_bootstrap_advancement_from_request(execution, |phase| {
        opened = Some(request.open_in_advancement(&phase));
    })
    .unwrap();
    let home = opened
        .unwrap()
        .unwrap_or_else(|refused| panic!("the actual predecessor initializes: {refused:?}"));
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the actual predecessor closes: {refused:?}"))
        .reclaim();
}

#[inline(never)]
fn refuse_successor(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
) {
    let request = fixture::request_with_handler_retirement_refusal(
        location.clone(),
        build,
        contacts.clone(),
        Arc::new(AtomicBool::new(false)),
    );
    let mut opened = None;
    with_bootstrap_advancement_from_request(execution, |phase| {
        opened = Some(request.open_in_advancement(&phase));
    })
    .unwrap();
    let refused = opened
        .unwrap()
        .err()
        .expect("a different authored program requires a transition");
    let requirement = refused
        .transition_required()
        .expect("the public Host exposes a typed transition requirement");
    assert_eq!(
        requirement.recorded().selected().identity(),
        "worth.query.host.home-identity.program.v1"
    );
    assert_eq!(
        requirement.requested().selected().identity(),
        "worth.query.host.home-retirement-refusal.program.v1"
    );
    assert_eq!(
        requirement.recorded().schema(),
        requirement.requested().schema()
    );
    assert_ne!(
        requirement.recorded().selected().revision(),
        requirement.requested().selected().revision()
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    refused.close(&execution).unwrap_or_else(|refused| {
        panic!("the refused successor disposes its actual preparation: {refused:?}")
    });
}

#[inline(never)]
fn reopen_predecessor(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
) {
    let request =
        fixture::request_with_initializer(location, build, contacts.clone(), |_, _, _| {
            panic!("the unchanged predecessor must reopen without its initializer")
        });
    let mut opened = None;
    with_bootstrap_advancement_from_request(execution, |phase| {
        opened = Some(request.open_in_advancement(&phase));
    })
    .unwrap();
    let home = opened.unwrap().unwrap_or_else(|refused| {
        panic!("the refusal preserved the original binding: {refused:?}")
    });
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the unchanged predecessor closes: {refused:?}"))
        .reclaim();
}
