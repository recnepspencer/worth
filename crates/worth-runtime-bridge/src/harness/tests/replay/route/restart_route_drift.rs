use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, ProfileTruth, ScenarioRelationalTruth};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

#[test]
fn bridge_replay_detects_route_drift_after_restart_shaped_truth_change() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;
    let truth = ProfileTruth::new(resource_request);
    let commit = truth.commit.clone();
    let fixture = ScenarioPlan::new(
        "bridge-replay-restart-drift",
        BridgeHarnessFixture::new(vec![ProfileTruth::mapping()]).with_relational_truth(
            ScenarioRelationalTruth::new(truth.native, truth.commit, vec![truth.observation], None),
        ),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit.clone()),
    );
    let profile = ExecutionProfile::development("development");

    let mut original = adapter.create_runtime().expect("bridge harness runtime");
    adapter
        .prepare_runtime(&mut original, &profile)
        .expect("bridge harness prepare");
    adapter
        .load_fixture(&mut original, &fixture)
        .expect("bridge harness load fixture");
    adapter
        .execute(&mut original, &fixture, &request, &profile)
        .expect("bridge harness execute");
    let original_record = original
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .diagnostics()
        .last_canonical_route_record()
        .expect("original canonical bridge route record");

    let drifted = ProfileTruth::on_branch_with_field(resource_request, "main", "status");
    assert_eq!(drifted.commit, commit);
    let drifted_fixture = ScenarioPlan::new(
        "bridge-replay-restart-drift-rehydrated",
        BridgeHarnessFixture::new(vec![ProfileTruth::mapping()]).with_relational_truth(
            ScenarioRelationalTruth::new(
                drifted.native,
                drifted.commit,
                vec![drifted.observation],
                None,
            ),
        ),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let mut restarted = adapter.create_runtime().expect("bridge harness runtime");
    adapter
        .prepare_runtime(&mut restarted, &profile)
        .expect("bridge harness prepare");
    adapter
        .load_fixture(&mut restarted, &drifted_fixture)
        .expect("bridge harness load fixture");
    let error = restarted
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .replay_canonical_record(&original_record, resource_request)
        .expect_err("bridge replay should reject route drift after restart");
    let original_route_record = original_record
        .decode()
        .expect("original canonical route record should decode");

    assert_eq!(
        error.kind(),
        crate::facade::BridgeReplayErrorKind::RouteMismatch
    );
    assert_eq!(
        error.context().route_identity(),
        Some(original_route_record.route_identity())
    );
    assert_eq!(
        error.context().snapshot_identity(),
        Some(original_route_record.source_snapshot())
    );
    let failure_record = restarted
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .diagnostics()
        .last_failure_record()
        .expect("bridge replay failure record");
    assert_eq!(failure_record.counters().route_replay_mismatch_count(), 1);
}
