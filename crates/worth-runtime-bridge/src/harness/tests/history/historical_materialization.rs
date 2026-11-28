use super::super::history_assertions::{assert_native_historical_record, last_historical_record};
use crate::facade::BridgeHistoricalMaterializationPath;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

#[test]
fn bridge_harness_executes_historical_commit_view() {
    let adapter = BridgeHarnessAdapter;
    let (truth, commit) = ScenarioRelationalTruth::named("alice", true);
    let snapshot = truth.snapshot().clone();
    let fixture = ScenarioPlan::new(
        "bridge-historical-commit-view",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
            commit.clone(),
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
    let _run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .expect("historical execution should succeed");

    let record = last_historical_record(&session);
    assert_native_historical_record(
        &record,
        &snapshot,
        &crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
        &commit,
        BridgeHistoricalMaterializationPath::CommitEnvelopeSnapshot,
    );
}

#[test]
fn bridge_harness_executes_branch_head_view() {
    let adapter = BridgeHarnessAdapter;
    let (truth, commit) = ScenarioRelationalTruth::named("alice", true);
    let snapshot = truth.snapshot().clone();
    let fixture = ScenarioPlan::new(
        "bridge-branch-head-view",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
    )
    .declare_input("branch-head:main")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "branch-head-main",
        BridgeHarnessTargetId::branch_head(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
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
    let _run = adapter
        .execute(&mut session, &fixture, &request, &profile)
        .expect("branch-head execution should succeed");

    let record = last_historical_record(&session);
    assert_native_historical_record(
        &record,
        &snapshot,
        &crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
        &commit,
        BridgeHistoricalMaterializationPath::BranchHeadEnvelopeSnapshot,
    );
}
