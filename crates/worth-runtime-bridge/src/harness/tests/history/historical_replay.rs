use super::super::history_assertions::{
    assert_historical_replay_summary, last_historical_record, replay_historical_record,
};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ReplayRequest, ScenarioPlan};
use worth_harness::runtime::{HarnessAdapter, ReplayHarnessAdapter};

#[test]
fn bridge_harness_replays_historical_record() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;

    let truth = NamedTruth::new();
    let (commit, observation) = truth.commit_name(resource_request, "alice");
    let head = truth.bind_main_head();
    let snapshot = head.snapshot_identity().clone();
    let fixture = ScenarioPlan::new(
        "bridge-historical-replay",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(
            ScenarioRelationalTruth::new(
                truth.native(),
                commit.clone(),
                vec![observation],
                Some(head),
            ),
        ),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
            commit,
        ),
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
        .expect("historical execution should succeed");
    let _replay = adapter
        .capture_replay(
            &session,
            &fixture,
            &ReplayRequest {
                name: "historical-replay".to_string(),
                source_run: run,
                request: request.clone(),
                profile: profile.clone(),
            },
        )
        .expect("historical replay capture should succeed");

    let record = last_historical_record(&session);
    let replay_summary = replay_historical_record(&session, &record, resource_request);
    assert_historical_replay_summary(&replay_summary, &record, &snapshot);
}

#[test]
fn bridge_harness_replays_historical_record_after_newer_publication_arrives() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;

    let truth = NamedTruth::new();
    let (commit, observation) = truth.commit_name(resource_request, "alice");
    let head = truth.bind_main_head();
    let snapshot = head.snapshot_identity().clone();
    let fixture = ScenarioPlan::new(
        "bridge-historical-replay-stability",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(
            ScenarioRelationalTruth::new(
                truth.native(),
                commit.clone(),
                vec![observation],
                Some(head),
            ),
        ),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
            commit,
        ),
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
        .expect("historical execution should succeed");

    let (_updated_commit, updated_observation) = truth.commit_name(resource_request, "bob");
    assert_ne!(updated_observation.snapshot_identity(), &snapshot);
    // Keep the original explicitly admitted Bridge head while Relational publishes newer truth.

    let _replay = adapter
        .capture_replay(
            &session,
            &fixture,
            &ReplayRequest {
                name: "historical-replay-after-newer-publication".to_string(),
                source_run: run,
                request,
                profile,
            },
        )
        .expect("historical replay should remain pinned to the original record");

    let record = last_historical_record(&session);
    let replay_summary = replay_historical_record(&session, &record, resource_request);
    assert_historical_replay_summary(&replay_summary, &record, &snapshot);
}
