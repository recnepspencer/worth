use super::*;

#[test]
fn unrelated_graph_scale_does_not_change_observation_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let fixture = authorization_fixture(execution);
    let baseline_snapshot = fixture
        .runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let baseline = fixture
        .runtime
        .observe_authorization(
            allow_plan(baseline_snapshot, fixture.principal, fixture.scope, []),
            execution,
        )
        .expect("baseline observation");
    for index in 0..64 {
        let left = create_entity(
            execution,
            &fixture.runtime,
            &format!("unrelated-left-{index}"),
        );
        let right = create_entity(
            execution,
            &fixture.runtime,
            &format!("unrelated-right-{index}"),
        );
        create_relation(
            execution,
            &fixture.runtime,
            left,
            right,
            &format!("unrelated-edge-{index}"),
        );
    }
    let scaled_snapshot = fixture
        .runtime
        .visibility_authority()
        .snapshot()
        .expect("current snapshot fixture is admitted");
    let scaled = fixture
        .runtime
        .observe_authorization(
            allow_plan(scaled_snapshot, fixture.principal, fixture.scope, []),
            execution,
        )
        .expect("scaled observation");

    assert_eq!(scaled.counters(), baseline.counters());
    assert_eq!(scaled.paths()[0].entities(), baseline.paths()[0].entities());
    assert_eq!(
        scaled.paths()[0].relations(),
        baseline.paths()[0].relations()
    );
}
