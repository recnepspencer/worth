use worth_harness::facade::{parity_suite, ExecutionProfile, ExecutionRequest, ScenarioPlan};

use crate::harness::adapter::BridgeHarnessTargetId;

use super::support::{
    build_runtime, committed_patch, field_aspect_registration, field_slice_snapshot, registration,
    snapshot,
};
use crate::facade::{
    BridgeBulkWorkloadRequest, BridgeBulkWorkloadSegment, BridgeContinuityAuthorityBasis,
    BridgeHistoricalLineageAuthority, BridgeHistoricalResolvedLineageIdentity,
    BridgeHistoricalResolvedRecordIdentity, BridgeLineageContext, BridgeRouteRequest,
    TruthBranchIdentity, TruthSnapshotIdentity,
};
use crate::harness::adapter::BridgeHarnessAdapter;
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::truth_identity_fixtures::{truth_branch, truth_commit, truth_patch, truth_snapshot};

fn continuity_authority(
    branch_identity: TruthBranchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
) -> BridgeHistoricalLineageAuthority {
    BridgeHistoricalLineageAuthority::try_new(
        BridgeContinuityAuthorityBasis::new(branch_identity, snapshot_identity),
        vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
            "lineage:test-successor",
        )],
        vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
            "entity:0:4:2",
        )],
        vec![7],
    )
    .expect("continuity authority should be canonical")
}

fn name_field_key() -> worth_foundational::facade::FieldKey {
    worth_foundational::facade::FieldKey::new("name".to_owned()).expect("valid harness field key")
}

#[test]
fn bridge_harness_parity_proves_routing_truth_is_invariant_across_diagnostics_tiers() {
    let fixture = ScenarioPlan::new(
        "bridge-parity",
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                commit_a(),
                patch_a(),
                snapshot_a(),
                name_field_key(),
            ))
            .with_snapshot(snapshot(snapshot_a(), "alice")),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit_a()),
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
    let fixture = ScenarioPlan::new(
        "bridge-fine-grained-parity",
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_aspect_mapping(field_aspect_registration())
            .with_committed_patch(committed_patch(
                commit_a(),
                patch_a(),
                snapshot_a(),
                name_field_key(),
            ))
            .with_snapshot(field_slice_snapshot(snapshot_a(), "alice")),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit_a()),
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
    let fixture = ScenarioPlan::new(
        "bridge-continuity-parity",
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_aspect_mapping(field_aspect_registration())
            .with_lineage_context(BridgeLineageContext::new(
                BridgeContinuityAuthorityBasis::new(main_branch(), snapshot_a()),
            ))
            .with_continuity_authority("user", continuity_authority(main_branch(), snapshot_a()))
            .with_committed_patch(committed_patch(
                commit_a(),
                patch_a(),
                snapshot_a(),
                name_field_key(),
            ))
            .with_snapshot(field_slice_snapshot(snapshot_a(), "alice")),
    )
    .declare_input("commit-a")
    .declare_observation("route")
    .compile();
    let request = ExecutionRequest::target(
        "deliver-commit-a",
        BridgeHarnessTargetId::committed_route(commit_a()),
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
    let fixture = ScenarioPlan::new(
        "bridge-historical-parity",
        BridgeHarnessFixture::new(vec![registration()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_committed_patch(committed_patch(
                commit_a(),
                patch_a(),
                snapshot_a(),
                name_field_key(),
            ))
            .with_snapshot(snapshot(snapshot_a(), "alice")),
    )
    .declare_input("history-commit:main:commit-a")
    .declare_observation("historical")
    .compile();
    let request = ExecutionRequest::target(
        "historical-commit-a",
        BridgeHarnessTargetId::historical_commit(main_branch(), commit_a()),
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

fn main_branch() -> TruthBranchIdentity {
    truth_branch("main")
}

fn commit_a() -> crate::facade::TruthCommitIdentity {
    truth_commit(1)
}

fn commit_b() -> crate::facade::TruthCommitIdentity {
    truth_commit(2)
}

fn patch_a() -> crate::facade::TruthPatchIdentity {
    truth_patch(1)
}

fn patch_b() -> crate::facade::TruthPatchIdentity {
    truth_patch(2)
}

fn snapshot_a() -> TruthSnapshotIdentity {
    truth_snapshot(1, 1)
}

fn snapshot_b() -> TruthSnapshotIdentity {
    truth_snapshot(2, 1)
}

mod bulk;
