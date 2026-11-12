//! A durable Home's actual World/Signal route starts cold after either build.
use std::sync::atomic::{AtomicUsize, Ordering};
use worth_store_testkit_application_oracle::{home_opening, world_home};
#[test]
fn reopened_home_recomputes_derived_signal_on_first_demand_under_same_and_changed_builds() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let expected = world_home::reference_signal_value(&execution);
    let calls = AtomicUsize::new(0);
    for (index, build) in [
        "11111111-1111-4111-8111-111111111111",
        "11111111-1111-4111-8111-111111111111",
        "22222222-2222-4222-8222-222222222222",
    ]
    .into_iter()
    .enumerate()
    {
        let opening = home_opening::request_with_build(
            directory.path(),
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
            worth_foundational::facade::ApplicationBuildIdentity::parse(build).unwrap(),
        );
        let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
        if index == 0 {
            drop(fixture.bootstrap_root(&execution));
            fixture.fork_derived_signal_branches(&execution);
        }
        assert_eq!(
            calls.load(Ordering::SeqCst),
            index * 3,
            "readmission invokes no evaluator"
        );
        fixture.assert_derived_signal_authored_reference(&execution);
        assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            (index + 1) * 3,
            "first demand recomputes"
        );
        assert_eq!(fixture.demand_derived_signal(&execution, &calls), expected);
        assert_eq!(
            calls.load(Ordering::SeqCst),
            (index + 1) * 3,
            "the serving cache remains reusable"
        );
        fixture.assert_derived_signal_authored_reference(&execution);
        let closed = fixture
            .home
            .close(&execution)
            .unwrap_or_else(|cause| panic!("actual Home close: {:?}", cause.reason()));
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
fn home_signal_descriptor_readmission_refuses_foreign_owner_and_unknown_codec() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let opening = home_opening::request(
        directory.path(),
        crate::workload::runtime_builder(crate::schema_transition::SOURCE),
    );
    let fixture = world_home::open_with_derived_signal(opening, &execution).unwrap();
    drop(fixture.bootstrap_root(&execution));
    fixture.assert_signal_descriptor_refusals(&execution);
    let closed = fixture
        .home
        .close(&execution)
        .unwrap_or_else(|cause| panic!("actual Home close: {:?}", cause.reason()));
    assert!(matches!(
        closed,
        worth_store_application::ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
}
