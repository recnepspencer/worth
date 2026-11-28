use super::super::history_assertions::last_historical_record;
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};
use worth_harness::facade::{ExecutionProfile, ExecutionRequest, ScenarioPlan};
use worth_harness::runtime::HarnessAdapter;

#[test]
fn bridge_harness_branch_divergence_changes_selected_truth_view_explicitly() {
    let adapter = BridgeHarnessAdapter;
    let profile = ExecutionProfile::development("development");
    let request = ExecutionRequest::target(
        "branch-head-feature",
        BridgeHarnessTargetId::branch_head(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("feature"),
        ),
    );

    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (main_commit, main_observation) = truth.commit_name(execution, "alice");
    let main_head = truth.bind_main_head();
    let main_snapshot = main_head.snapshot_identity().clone();
    truth.fork_main(execution, "feature");
    let (_feature_commit, feature_observation) =
        truth.commit_name_on_branch(execution, "feature", "frank");
    let feature_head = truth.bind_head("feature");
    let feature_snapshot = feature_head.snapshot_identity().clone();
    let fixture = ScenarioPlan::new(
        "bridge-historical-branch-heads",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(
            ScenarioRelationalTruth::new_with_heads(
                truth.native(),
                main_commit,
                vec![main_observation, feature_observation],
                vec![main_head, feature_head],
            ),
        ),
    )
    .declare_input("branch-head:main")
    .declare_input("branch-head:feature")
    .declare_observation("historical")
    .compile();

    let mut main_session = adapter
        .create_runtime()
        .expect("main bridge harness runtime");
    adapter
        .prepare_runtime(&mut main_session, &profile)
        .expect("main bridge harness prepare");
    adapter
        .load_fixture(&mut main_session, &fixture)
        .expect("main bridge harness load fixture");
    let _main_run = adapter
        .execute(
            &mut main_session,
            &fixture,
            &ExecutionRequest::target(
                "branch-head-main",
                BridgeHarnessTargetId::branch_head(
                    crate::facade::TruthBranchIdentity::from_relational_branch_id("main"),
                ),
            ),
            &profile,
        )
        .expect("main branch-head execution should succeed");

    let mut feature_session = adapter
        .create_runtime()
        .expect("feature bridge harness runtime");
    adapter
        .prepare_runtime(&mut feature_session, &profile)
        .expect("feature bridge harness prepare");
    adapter
        .load_fixture(&mut feature_session, &fixture)
        .expect("feature bridge harness load fixture");
    let _feature_run = adapter
        .execute(&mut feature_session, &fixture, &request, &profile)
        .expect("feature branch-head execution should succeed");

    let main_record = last_historical_record(&main_session);
    let feature_record = last_historical_record(&feature_session);
    assert_eq!(
        main_record.decision_log().snapshot_identity(),
        &main_snapshot
    );
    assert_eq!(
        feature_record.decision_log().snapshot_identity(),
        &feature_snapshot
    );
    assert_ne!(
        main_record.record_identity(),
        feature_record.record_identity()
    );
    assert_ne!(
        main_record.decision_log().snapshot_identity(),
        feature_record.decision_log().snapshot_identity()
    );
    assert_ne!(
        main_record.decision_log().branch_identity(),
        feature_record.decision_log().branch_identity()
    );
}
