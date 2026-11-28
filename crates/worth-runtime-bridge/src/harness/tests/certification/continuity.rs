use worth_harness::facade::{
    certification_matrix, ExecutionProfile, ExecutionRequest, HarnessAdapter, ScenarioPlan,
};

use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, ProfileTruth};

fn continuity_fixture(
    name: &str,
    branch: &str,
    ambiguous: bool,
) -> (
    worth_harness::facade::ScenarioFixture<BridgeHarnessFixture>,
    BridgeHarnessTargetId,
) {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = ProfileTruth::on_branch_with_field(execution, branch, "title");
    let target = BridgeHarnessTargetId::committed_route(truth.commit.clone());
    let fixture = BridgeHarnessFixture::new(vec![ProfileTruth::mapping()])
        .with_aspect_mapping(ProfileTruth::aspect_mapping())
        .with_lineage_context(truth.lineage_context())
        .with_relational_truth(truth.into_scenario());
    let fixture = if ambiguous {
        fixture.with_ambiguous_lineage()
    } else {
        fixture
    };
    (
        ScenarioPlan::new(name, fixture)
            .declare_input("profile-update")
            .declare_observation("route")
            .compile(),
        target,
    )
}

#[test]
fn bridge_continuity_certification_matrix_reports_candidate_profile_parity() {
    let (fixture, target) = continuity_fixture("bridge-continuity-certification", "main", false);
    let request = ExecutionRequest::target("deliver-profile-update", target);

    let report = certification_matrix(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([ExecutionProfile::forensic("forensic")])
    .certify()
    .expect("bridge continuity certification matrix should succeed");

    assert!(report.matched);
    assert!(report.baseline_diagnostics_summary.is_some());
    assert_eq!(report.cases.len(), 1);
}

#[test]
fn bridge_harness_branch_divergence_changes_terminal_continuity_export() {
    let adapter = BridgeHarnessAdapter;
    let profile = ExecutionProfile::development("baseline");
    let (main_fixture, main_target) = continuity_fixture("bridge-continuity-main", "main", false);
    let (feature_fixture, feature_target) =
        continuity_fixture("bridge-continuity-feature", "feature", false);
    let main_request = ExecutionRequest::target("deliver-profile-update", main_target);
    let feature_request = ExecutionRequest::target("deliver-profile-update", feature_target);

    let mut main_runtime = adapter.create_runtime().expect("main harness runtime");
    adapter
        .prepare_runtime(&mut main_runtime, &profile)
        .expect("main harness prepare");
    adapter
        .load_fixture(&mut main_runtime, &main_fixture)
        .expect("main harness load fixture");
    let main_run = adapter
        .execute(&mut main_runtime, &main_fixture, &main_request, &profile)
        .expect("main harness execute");

    let mut feature_runtime = adapter.create_runtime().expect("feature harness runtime");
    adapter
        .prepare_runtime(&mut feature_runtime, &profile)
        .expect("feature harness prepare");
    adapter
        .load_fixture(&mut feature_runtime, &feature_fixture)
        .expect("feature harness load fixture");
    let feature_run = adapter
        .execute(
            &mut feature_runtime,
            &feature_fixture,
            &feature_request,
            &profile,
        )
        .expect("feature harness execute");

    assert_ne!(main_run.summary, feature_run.summary);
    assert_ne!(main_run.extensions, feature_run.extensions);
    assert!(main_run.extensions.contains_key("bridge_continuity_record"));
    assert!(feature_run
        .extensions
        .contains_key("bridge_continuity_record"));
}

#[test]
fn bridge_harness_continuity_exports_ambiguous_rejection_record() {
    let adapter = BridgeHarnessAdapter;
    let profile = ExecutionProfile::development("baseline");
    let (fixture, target) = continuity_fixture("bridge-continuity-ambiguous", "main", true);
    let request = ExecutionRequest::target("deliver-profile-update", target);

    let mut runtime = adapter.create_runtime().expect("harness runtime");
    adapter
        .prepare_runtime(&mut runtime, &profile)
        .expect("harness prepare");
    adapter
        .load_fixture(&mut runtime, &fixture)
        .expect("harness load fixture");
    let run = adapter
        .execute(&mut runtime, &fixture, &request, &profile)
        .expect("harness execute");

    assert!(run.summary.is_object());
    assert!(run.extensions.contains_key("bridge_continuity_record"));
}
