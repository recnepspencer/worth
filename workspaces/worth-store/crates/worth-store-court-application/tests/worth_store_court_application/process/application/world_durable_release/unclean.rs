//! The happy-track gate refuses a torn owner/product operation, not a quiet Home.
use super::*;
use worth_query_host::facade::{CanonicalDigestId, DurableOwnerFamily};
use worth_store_application::ApplicationHomeOpenRefusal;
use worth_store_testkit_application_oracle::world_home::WorldHomeFixtureRefusal;
mod quiescent;

#[test]
fn unclean_native_tail_refuses_before_world_service_and_names_the_original_effect() {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    let markers = root.path().join("markers");
    std::fs::create_dir(&markers).unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[(LOCATION, location.as_path()), (MARKERS, markers.as_path())],
            &markers.join("ready"),
            &markers.join("finished"),
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let refusal = match worth_store_testkit_application_oracle::world_home::open(
        worth_store_testkit_application_oracle::home_opening::request(
            &location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        &execution,
    ) {
        Err(WorldHomeFixtureRefusal::World(refusal)) => refusal,
        Err(other) => panic!("actual unclean-tail refusal: {other:?}"),
        Ok(_) => panic!("an unclean Native tail cannot issue a World-reconciled Home"),
    };
    let observed = match refusal.reason() {
        ApplicationHomeOpenRefusal::UncleanRecoveryUnsupported(observed) => *observed,
        other => panic!("typed unclean-open refusal required: {other:?}"),
    };
    assert_eq!(observed.owner, DurableOwnerFamily::Relational);
    let retained = refusal.into_home();
    let subjects = retained
        .relational()
        .runtime()
        .durability_authority()
        .discover_all_associated_effects(&execution)
        .unwrap();
    let [original] = subjects.as_slice() else {
        panic!("one actual Native owner effect");
    };
    assert_eq!(
        observed.effect,
        CanonicalDigestId::new(*original.artifact_digest())
    );
}

#[test]
fn quiescent_kill_reopens_the_acknowledged_native_and_world_heads() {
    quiescent::journey(quiescent::End::Killed);
}

#[test]
fn graceful_close_reopens_the_checkpointed_native_and_world_heads() {
    quiescent::journey(quiescent::End::Closed);
}
