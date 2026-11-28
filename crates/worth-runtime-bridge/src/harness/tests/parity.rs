use worth_harness::facade::{parity_suite, ExecutionProfile, ExecutionRequest, ScenarioPlan};

use super::support::build_runtime;
use crate::facade::{BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeRouteRequest};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessTargetId};
use crate::harness::fixtures::{
    BridgeHarnessFixture, NamedTruth, ProfileTruth, ScenarioRelationalTruth,
};

#[test]
fn bridge_harness_parity_proves_routing_truth_is_invariant_across_diagnostics_tiers() {
    let (truth, commit) = ScenarioRelationalTruth::named("alice", false);
    let fixture = ScenarioPlan::new(
        "bridge-parity",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit),
    );

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([
        ExecutionProfile::operational("operational"),
        ExecutionProfile::forensic("forensic"),
    ])
    .compare()
    .expect("bridge parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 2);
}

#[test]
fn bridge_harness_parity_proves_fine_grained_slice_truth_is_invariant_across_diagnostics_tiers() {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = ProfileTruth::new(execution);
    let commit = truth.commit.clone();
    let fixture = ScenarioPlan::new(
        "bridge-fine-grained-parity",
        BridgeHarnessFixture::new(vec![ProfileTruth::mapping()])
            .with_aspect_mapping(ProfileTruth::aspect_mapping())
            .with_relational_truth(truth.into_scenario()),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit),
    );

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([
        ExecutionProfile::operational("operational"),
        ExecutionProfile::forensic("forensic"),
    ])
    .compare()
    .expect("fine-grained bridge parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 2);
}

#[test]
fn bridge_harness_parity_proves_continuity_truth_is_invariant_across_diagnostics_tiers() {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = ProfileTruth::new(execution);
    let commit = truth.commit.clone();
    let fixture = ScenarioPlan::new(
        "bridge-continuity-parity",
        BridgeHarnessFixture::new(vec![ProfileTruth::mapping()])
            .with_aspect_mapping(ProfileTruth::aspect_mapping())
            .with_lineage_context(truth.lineage_context())
            .with_relational_truth(truth.into_scenario()),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit),
    );

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([
        ExecutionProfile::operational("operational"),
        ExecutionProfile::forensic("forensic"),
    ])
    .compare()
    .expect("continuity parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 2);
}

#[test]
fn bridge_harness_parity_proves_historical_truth_is_invariant_across_diagnostics_tiers() {
    let (truth, commit) = ScenarioRelationalTruth::named("alice", true);
    let fixture = ScenarioPlan::new(
        "bridge-historical-parity",
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()]).with_relational_truth(truth),
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

    let report = parity_suite(
        BridgeHarnessAdapter,
        fixture,
        request,
        ExecutionProfile::development("baseline"),
    )
    .candidates([
        ExecutionProfile::operational("operational"),
        ExecutionProfile::forensic("forensic"),
    ])
    .compare()
    .expect("historical parity suite should compare cleanly");

    assert!(report.matched);
    assert_eq!(report.results.len(), 2);
}

mod bulk;
