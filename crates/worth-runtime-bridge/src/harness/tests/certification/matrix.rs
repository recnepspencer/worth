use worth_harness::facade::{
    certification_matrix, ExecutionProfile, ExecutionRequest, ScenarioPlan,
};

use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth, ScenarioRelationalTruth};

#[test]
fn bridge_certification_matrix_reports_diagnostics_for_candidate_profiles() {
    let (truth, commit) = ScenarioRelationalTruth::named("alice", false);
    let fixture = ScenarioPlan::new(
        "bridge-certification",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit.clone()),
    );

    let report = certification_matrix(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([ExecutionProfile::forensic("forensic")])
    .certify()
    .expect("bridge certification matrix should succeed");

    assert!(report.matched);
    assert!(report.baseline_diagnostics_summary.is_some());
    assert_eq!(report.cases.len(), 1);
}

#[test]
fn bridge_historical_certification_matrix_reports_candidate_profile_parity() {
    let (truth, commit) = ScenarioRelationalTruth::named("alice", true);
    let fixture = ScenarioPlan::new(
        "bridge-historical-certification",
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

    let report = certification_matrix(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([ExecutionProfile::forensic("forensic")])
    .certify()
    .expect("bridge historical certification matrix should succeed");

    assert!(report.matched);
    assert!(report.baseline_diagnostics_summary.is_some());
    assert_eq!(report.cases.len(), 1);
}
