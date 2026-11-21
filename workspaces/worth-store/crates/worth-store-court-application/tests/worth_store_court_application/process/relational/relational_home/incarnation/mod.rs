//! Open identity, incarnation, pre-effect refusals and independent OS contention.
use crate::durable_run::open;
#[allow(unused_imports)]
use crate::incarnation::*;
use worth_store_testkit_application_oracle::facade::home_opening::HomeOpeningRefused as OpenRefusal;

#[test]
fn contention_observer_child() {
    let Some(location) = std::env::var_os("WORTH_INCARNATION_OBSERVE") else {
        return;
    };
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    assert!(
        matches!(open(std::path::Path::new(&location), &execution), Err(OpenRefusal::BeforeReadmission(worth_store_application::ApplicationHomeOpenRefusal::Physical(refusal))) if refusal.kind() == worth_store_application::ApplicationHomePhysicalRefusalKind::MutationOwnerContended)
    );
}

#[test]
fn home_identity_is_stable_and_each_successful_open_advances_once() {
    let parent = tempfile::tempdir().unwrap();
    let location = parent.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let first = open(&location, &execution).unwrap();
    assert_eq!(first.incarnation().get(), 1);
    let identity = first.home_identity();
    first.close(&execution);
    for expected in 2..=4 {
        let home = open(&location, &execution).unwrap();
        assert_eq!(home.home_identity(), identity);
        assert_eq!(home.incarnation().get(), expected);
        home.close(&execution);
    }
}

#[path = "anchor_replay/mod.rs"]
pub(crate) mod anchor_replay;
#[path = "media/mod.rs"]
pub(crate) mod media;
mod refusals;

#[path = "writable_ownership/mod.rs"]
pub(crate) mod writable_ownership;
