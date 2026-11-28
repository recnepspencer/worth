use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ReplayRequest, ScenarioPlan};
use worth_harness::runtime::{HarnessAdapter, ReplayHarnessAdapter};

#[test]
fn bridge_replay_capture_exposes_last_route_record() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;
    let (truth, commit) = ScenarioRelationalTruth::named("alice", false);
    let snapshot = truth.snapshot().clone();
    let fixture = ScenarioPlan::new(
        "bridge-replay",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit.clone()),
    );
    let profile = ExecutionProfile::development("development");

    let mut session = adapter.create_runtime().expect("bridge harness runtime");
    adapter
        .prepare_runtime(&mut session, &profile)
        .expect("bridge harness prepare");
    adapter
        .load_fixture(&mut session, &fixture)
        .expect("bridge harness load fixture");
    let run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .expect("bridge harness execute");
    let canonical_record = session
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .diagnostics()
        .last_canonical_route_record()
        .expect("bridge should retain typed route record before replay capture");
    let replay = adapter
        .capture_replay(
            &session,
            &fixture,
            &ReplayRequest {
                name: "replay".to_string(),
                source_run: run,
                request: request.clone(),
                profile: profile.clone(),
            },
        )
        .expect("bridge replay capture should succeed");
    let typed_replay = session
        .runtime
        .as_ref()
        .expect("bridge runtime")
        .replay_canonical_record(&canonical_record, resource_request)
        .expect("typed route replay should succeed from retained route record");

    assert_eq!(
        typed_replay.source_commit().canonical_label(),
        commit.clone().canonical_label()
    );
    assert_eq!(
        typed_replay.source_snapshot().canonical_label(),
        snapshot.clone().canonical_label()
    );
    assert_eq!(replay.requested_targets, request.targets);
}
