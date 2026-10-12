#[path = "support/test_execution_policy.rs"]
mod test_execution_policy;
use worth_query::facade::runtime::{
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
};
use worth_server::{
    WorthServerQueryDependencyAuditPathKind, WorthServerQueryDependencyClosurePosture,
    WorthServerQueryDependencyConsumerKitPosture, WorthServerQueryDependencyRuntimeReadiness,
};

#[path = "support/query_handoff/fixture.rs"]
mod query_handoff_fixture;
#[path = "support/query_handoff/runtime.rs"]
mod query_handoff_runtime;

use query_handoff_fixture::test_server;
use query_handoff_runtime::{ProfiledTestWorkspaceProvider, TestWorkspaceProvider};

#[test]
fn server_query_dependency_audit_consumes_production_owned_covered_path_inventory() {
    let server = test_server(TestWorkspaceProvider, false);

    let receipt = server.query_dependency_audit().run();
    let inventory = receipt.covered_path_inventory();
    let row_ids = receipt
        .rows()
        .iter()
        .map(|row| row.row_id().as_str())
        .collect::<Vec<_>>();

    assert_eq!(receipt.rows().len(), inventory.row_ids().len());
    assert_eq!(
        inventory.ordinary_row_count(),
        receipt
            .rows()
            .iter()
            .filter(|row| row.ordinary_path())
            .count()
    );
    assert_eq!(
        inventory.static_test_only_row_count(),
        receipt
            .rows()
            .iter()
            .filter(|row| !row.ordinary_path())
            .count()
    );
    assert_eq!(inventory.inventory_digest(), inventory.row_ids().join("|"));
    assert_eq!(
        row_ids,
        inventory
            .row_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
    assert_eq!(inventory.static_test_only_row_count(), 0);
    assert!(receipt.rows().iter().all(|row| !row.reason().is_empty()));
    assert!(!receipt.audit_digest().is_empty());
    assert!(receipt.ordinary_rows().iter().all(|row| row.scope_posture()
        != worth_server::WorthServerQueryDependencyScopePosture::Unclassified));
}

#[test]
fn server_paths_expose_real_query_support_pinning_provenance() {
    let server = test_server(TestWorkspaceProvider, false);

    let receipt = server.query_dependency_audit().run();

    for path_kind in [
        WorthServerQueryDependencyAuditPathKind::WorthNativeDirectRead,
        WorthServerQueryDependencyAuditPathKind::WorthNativeDirectState,
        WorthServerQueryDependencyAuditPathKind::WorthNativeDirectInspection,
        WorthServerQueryDependencyAuditPathKind::WorthNativeDirectProjection,
        WorthServerQueryDependencyAuditPathKind::CompatibilityHttpRead,
        WorthServerQueryDependencyAuditPathKind::QueryHandoffRead,
        WorthServerQueryDependencyAuditPathKind::QueryHandoffDownstreamDelivery,
    ] {
        let row = receipt
            .row(path_kind)
            .expect("read-shaped audit row should exist");
        let provenance = row
            .provenance()
            .support_pin()
            .expect("ordinary path should expose support pin provenance");
        assert_eq!(
            row.runtime_readiness(),
            WorthServerQueryDependencyRuntimeReadiness::QueryNineSevenSharedReadClosureReady
        );
        assert_eq!(
            row.closure_posture(),
            WorthServerQueryDependencyClosurePosture::Ready
        );
        assert_eq!(provenance.workspace_name(), "audit-workspace");
        assert!(!provenance.support_matrix_digest().is_empty());
        assert!(!provenance.support_snapshot_digest().is_empty());
        assert!(!provenance.contract_digest().is_empty());
        assert!(!provenance.report_digest().is_empty());
        assert_eq!(provenance.blocking_finding_count(), 0);
    }

    for path_kind in [
        WorthServerQueryDependencyAuditPathKind::WorthNativeDirectMutation,
        WorthServerQueryDependencyAuditPathKind::CompatibilityHttpMutation,
        WorthServerQueryDependencyAuditPathKind::QueryHandoffMutation,
    ] {
        let row = receipt
            .row(path_kind)
            .expect("mutation-shaped audit row should exist");
        let provenance = row
            .provenance()
            .support_pin()
            .expect("mutation path should expose support pin provenance");
        assert_eq!(
            row.runtime_readiness(),
            WorthServerQueryDependencyRuntimeReadiness::QueryNineSevenDeterministicSubmissionClosureReady
        );
        assert_eq!(
            row.closure_posture(),
            WorthServerQueryDependencyClosurePosture::Ready
        );
        assert_eq!(
            provenance.required_families(),
            &[
                WorthQueryRuntimeFacadeFamily::Write,
                WorthQueryRuntimeFacadeFamily::Submission,
                WorthQueryRuntimeFacadeFamily::Inspect,
            ]
        );
        assert_eq!(provenance.matched_required_count(), 3);
    }
}

#[test]
fn server_support_posture_uses_query_runtime_receipts() {
    let server = test_server(TestWorkspaceProvider, false);

    let receipt = server.query_dependency_audit().run();

    assert!(receipt.ordinary_rows().iter().all(|row| {
        row.consumer_kit_posture()
            == WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningAdopted
    }));
    assert_eq!(receipt.ordinary_rows().len(), receipt.rows().len());
}

#[test]
fn hostile_support_profile_blocks_operation_runtime_closure_with_real_pin_provenance() {
    let server = test_server(
        ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                WorthQueryRuntimeFamilySupport::unsupported(
                    WorthQueryRuntimeFacadeFamily::SharedRead,
                    "phase-one hostile audit fixture",
                ),
            ),
        ),
        false,
    );

    let receipt = server.query_dependency_audit().run();
    let row = receipt
        .row(WorthServerQueryDependencyAuditPathKind::WorthNativeDirectRead)
        .expect("read row should exist");
    let provenance = row
        .provenance()
        .support_pin()
        .expect("blocked row should still preserve pin provenance");

    assert!(!receipt.is_runtime_ready_for_phase_one());
    assert!(!receipt
        .rows_with_closure_posture(WorthServerQueryDependencyClosurePosture::Blocked)
        .is_empty());
    assert!(receipt.support_posture().blocked_row_count() > 0);
    assert_eq!(
        row.consumer_kit_posture(),
        WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningBlocked
    );
    assert_eq!(provenance.workspace_name(), "audit-workspace");
    assert!(provenance.blocking_finding_count() > 0);
}
