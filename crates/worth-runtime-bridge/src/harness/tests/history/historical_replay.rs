use super::super::history_assertions::{
    assert_historical_replay_summary, last_historical_record, replay_historical_record,
};
use super::super::support::{committed_patch, registration, snapshot};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessMutation, BridgeHarnessTargetId};
use crate::harness::fixtures::BridgeHarnessFixture;
use worth_harness::facade::{
    ExecutionProfile, ExecutionRequest, MutationBatch, ReplayRequest, ScenarioPlan,
};
use worth_harness::runtime::{HarnessAdapter, ReplayHarnessAdapter};

#[test]
fn bridge_harness_replays_historical_record() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "bridge-historical-replay",
        BridgeHarnessFixture::new(vec![registration()])
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
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
    assert_historical_replay_summary(&replay_summary, &record, "snapshot-a");
}

#[test]
fn bridge_harness_replays_historical_record_after_newer_publication_arrives() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let adapter = BridgeHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "bridge-historical-replay-stability",
        BridgeHarnessFixture::new(vec![registration()])
            .with_committed_patch(committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ))
            .with_snapshot(snapshot(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                "alice",
            )),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
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

    let mutation = MutationBatch::new("publish-newer-history")
        .push(BridgeHarnessMutation::PublishCommittedPatch(
            committed_patch(
                crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
                crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid harness field key"),
            ),
        ))
        .push(BridgeHarnessMutation::PublishSnapshot(snapshot(
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
            "bob",
        )));
    adapter
        .apply_mutation_batch(&mut session, &mutation)
        .expect("mutation batch should apply");

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
    assert_historical_replay_summary(&replay_summary, &record, "snapshot-a");
}
