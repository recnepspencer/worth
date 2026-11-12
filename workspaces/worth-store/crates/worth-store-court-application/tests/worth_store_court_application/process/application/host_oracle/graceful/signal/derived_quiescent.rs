//! The quiet child has acknowledged both World/Signal effects before it parks.
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_store_testkit_application_oracle::{home_opening, world_home};
const LOCATION: &str = "WORTH_QUIET_SIGNAL_HOME";
const MARKERS: &str = "WORTH_QUIET_SIGNAL_MARKERS";
const CHILD:&str="process::application::host_oracle::graceful::signal::derived_quiescent::quiet_signal_home_value_child";
#[test]
fn quiescent_home_kill_recomputes_derived_signal_only_on_demand_under_same_and_changed_builds() {
    for build in [super::FIRST, super::SECOND] {
        let root = tempfile::tempdir().unwrap();
        let location = root.path().join("home");
        let markers = root.path().join("markers");
        std::fs::create_dir(&markers).unwrap();
        assert_eq!(
            crate::killed_child::run_and_kill(
                CHILD,
                &[(LOCATION, location.as_path()), (MARKERS, markers.as_path())],
                &markers.join("ready"),
                &markers.join("finished")
            ),
            crate::killed_child::ChildEnd::KilledAtPark
        );
        let request = crate::workload::serial_request();
        let execution = ExecutionRequest::serial(&request);
        let opening = home_opening::request_with_build(
            &location,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            worth_foundational::facade::ApplicationBuildIdentity::parse(build).unwrap(),
        );
        let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
        let calls = AtomicUsize::new(0);
        fixture.assert_derived_signal_authored_reference(&execution);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let expected = world_home::reference_signal_value(&execution);
        assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
        assert_eq!(calls.load(Ordering::SeqCst), 3);
        let closed = fixture.home.close(&execution).unwrap_or_else(|cause| {
            panic!("actual recovered Signal Home close: {:?}", cause.reason())
        });
        assert!(matches!(
            closed,
            worth_store_application::ApplicationHomeCloseOutcome::Closed {
                checkpoint: Ok(_),
                ..
            }
        ));
    }
}
#[test]
fn quiet_signal_home_value_child() {
    let Some(location) = std::env::var_os(LOCATION).map(std::path::PathBuf::from) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let opening = home_opening::request(
        location,
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
    drop(fixture.bootstrap_root(&execution));
    fixture.fork_derived_signal_branches(&execution);
    let calls = AtomicUsize::new(0);
    let expected = world_home::reference_signal_value(&execution);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    fixture.assert_derived_signal_authored_reference(&execution);
    crate::killed_child::announce(
        &markers.join("ready"),
        b"quiescent after original Signal publications",
    );
    crate::killed_child::park();
}
