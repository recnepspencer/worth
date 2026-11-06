//! Destructive Host disposal uses the same filesystem lease as Store Application.
mod interruption;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_query_host::facade::{
    discard_home, WorthQueryApplicationHomeDiscardDenial as Denial,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusalKind};

fn discard(path: &Path) -> Result<(), Denial> {
    discard_home(Location::Directory(path.to_owned()))
}

fn close(
    home: worth_store_application::ReadmittedApplicationHome,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    match home.close(execution) {
        worth_store_application::ApplicationHomeCloseOutcome::Closed { .. } => {}
        worth_store_application::ApplicationHomeCloseOutcome::CloseRefused { reason, .. } => {
            panic!("disposal prerequisite requires actual Closed: {reason:?}")
        }
    }
}

#[test]
fn discard_refuses_the_live_lease_and_closed_disposal_reinitializes_with_a_new_identity() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(&location, &execution).unwrap();
    let identity = home.home_identity();
    assert!(matches!(
        discard(&location),
        Err(Denial::MutationOwnerContended)
    ));
    assert!(!location.join("discard.pending").exists());
    close(home, &execution);
    discard(&location).unwrap();
    assert!(!location.join("namespace/identity").exists());
    assert!(!location.join("discard.pending").exists());
    assert!(location.join("namespace/mutation.lock").is_file());
    let new = crate::durable_run::open(&location, &execution).unwrap();
    assert_ne!(new.home_identity(), identity);
    assert_eq!(new.incarnation().get(), 1);
    close(new, &execution);
    discard(&location).unwrap();
    discard(&location).unwrap();
    assert_fresh_host(&location);
}

fn assert_fresh_host(location: &Path) {
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement_from_request,
        application_installation::ApplicationBuildIdentity,
    };
    use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
    let initialized = Arc::new(AtomicUsize::new(0));
    let input = fixture::request(
        location.to_owned(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        initialized.clone(),
        Arc::new(AtomicUsize::new(0)),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        input.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("discarded Home must initialize through Host: {refused:?}"));
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert!(matches!(
        discard(location),
        Err(Denial::MutationOwnerContended)
    ));
    // This fixture's opaque handler cannot yet retire (P-B). Abandonment is not Closed.
    drop(home);
}

#[test]
fn discard_completes_damaged_store_and_namespace_identity_without_store_recovery() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(&location, &execution).unwrap();
    let identity = home.home_identity();
    close(home, &execution);
    std::fs::write(location.join("namespace/identity"), b"damaged identity").unwrap();
    std::fs::write(location.join("families/damaged-payload"), b"torn store").unwrap();
    let outside = directory.path().join("outside");
    std::fs::write(&outside, b"unrelated bytes").unwrap();
    std::fs::hard_link(&outside, location.join("families/linked-payload")).unwrap();
    discard(&location).unwrap();
    assert_eq!(std::fs::read(&outside).unwrap(), b"unrelated bytes");
    let new = crate::durable_run::open(&location, &execution).unwrap();
    assert_ne!(new.home_identity(), identity);
    close(new, &execution);
}

#[test]
fn damaged_tombstone_directory_is_completed_without_losing_the_opening_fence() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    std::fs::create_dir_all(location.join("discard.pending/partial")).unwrap();
    std::fs::write(location.join("discard.pending/partial/torn"), b"damage").unwrap();
    assert_discard_pending(&location);
    discard(&location).unwrap();
    assert!(!location.join("discard.pending").exists());
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    close(
        crate::durable_run::open(&location, &execution).unwrap(),
        &execution,
    );
}

fn assert_discard_pending(location: &Path) {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let physical = worth_store_testkit_application_oracle::facade::home_opening::request(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    match worth_store_application::recover_home(physical, &execution) {
        Err(ApplicationHomeOpenRefusal::Physical(refused)) => assert_eq!(
            refused.kind(),
            ApplicationHomePhysicalRefusalKind::DiscardPending
        ),
        Err(cause) => panic!("expected the tombstone fence: {cause:?}"),
        Ok(_) => panic!("a tombstoned Home must never open"),
    }
}

#[test]
fn torn_discard_intent_refuses_open_and_explicit_retry_finishes_it() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    std::fs::create_dir(&location).unwrap();
    std::fs::write(location.join("discard.pending"), b"torn").unwrap();
    assert_discard_pending(&location);
    assert_host_discard_pending(&location);
    discard(&location).unwrap();
    assert_fresh_host(&location);
}

fn assert_host_discard_pending(location: &Path) {
    use worth_query_host::facade::{
        application_contribution::with_bootstrap_advancement_from_request,
        application_installation::ApplicationBuildIdentity, WorthQueryApplicationHomeOpenRefused,
    };
    use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
    let initialized = Arc::new(AtomicUsize::new(0));
    let input = fixture::request(
        location.to_owned(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        initialized.clone(),
        Arc::new(AtomicUsize::new(0)),
    );
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    match with_bootstrap_advancement_from_request(execution, |phase| {
        input.open_in_advancement(&phase)
    })
    .unwrap()
    {
        Err(WorthQueryApplicationHomeOpenRefused::Physical {
            cause: ApplicationHomeOpenRefusal::Physical(cause),
            ..
        }) => assert_eq!(
            cause.kind(),
            ApplicationHomePhysicalRefusalKind::DiscardPending
        ),
        Err(cause) => panic!("Host must preserve the typed tombstone refusal: {cause:?}"),
        Ok(_) => panic!("Host cannot serve a tombstoned location"),
    }
    assert_eq!(initialized.load(Ordering::SeqCst), 0);
}
