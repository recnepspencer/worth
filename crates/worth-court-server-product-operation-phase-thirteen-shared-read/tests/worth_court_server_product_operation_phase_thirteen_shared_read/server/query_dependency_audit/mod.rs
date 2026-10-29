#[allow(unused_imports)]
use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};
use {worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily, worth_query::facade::runtime::WorthQueryRuntimeFamilySupport, worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
use {worth_server::WorthServerQueryDependencyAuditPathKind, worth_server::WorthServerQueryDependencyClosurePosture, worth_server::WorthServerQueryDependencyConsumerKitPosture, worth_server::WorthServerQueryDependencyRuntimeReadiness};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_query_handoff_fixture as query_handoff_fixture};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};

use {worth_testkit_server::shared_support_query_handoff_fixture::test_server};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider, worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::TestWorkspaceProvider};

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
        worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectRead,
        worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectState,
        worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectInspection,
        worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectProjection,
        worth_server::WorthServerQueryDependencyAuditPathKind::CompatibilityHttpRead,
        worth_server::WorthServerQueryDependencyAuditPathKind::QueryHandoffRead,
        worth_server::WorthServerQueryDependencyAuditPathKind::QueryHandoffDownstreamDelivery,
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
            worth_server::WorthServerQueryDependencyRuntimeReadiness::QueryNineSevenSharedReadClosureReady
        );
        assert_eq!(
            row.closure_posture(),
            worth_server::WorthServerQueryDependencyClosurePosture::Ready
        );
        assert_eq!(provenance.workspace_name(), "audit-workspace");
        assert!(!provenance.support_matrix_digest().is_empty());
        assert!(!provenance.support_snapshot_digest().is_empty());
        assert!(!provenance.contract_digest().is_empty());
        assert!(!provenance.report_digest().is_empty());
        assert_eq!(provenance.blocking_finding_count(), 0);
    }

    for path_kind in [
        worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectMutation,
        worth_server::WorthServerQueryDependencyAuditPathKind::CompatibilityHttpMutation,
        worth_server::WorthServerQueryDependencyAuditPathKind::QueryHandoffMutation,
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
            worth_server::WorthServerQueryDependencyRuntimeReadiness::QueryNineSevenDeterministicSubmissionClosureReady
        );
        assert_eq!(
            row.closure_posture(),
            worth_server::WorthServerQueryDependencyClosurePosture::Ready
        );
        assert_eq!(
            provenance.required_families(),
            &[
                worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Write,
                worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Submission,
                worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Inspect,
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
            == worth_server::WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningAdopted
    }));
    assert_eq!(receipt.ordinary_rows().len(), receipt.rows().len());
}

#[test]
fn hostile_support_profile_blocks_operation_runtime_closure_with_real_pin_provenance() {
    let server = test_server(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::SharedRead,
                    "phase-one hostile audit fixture",
                ),
            ),
        ),
        false,
    );

    let receipt = server.query_dependency_audit().run();
    let row = receipt
        .row(worth_server::WorthServerQueryDependencyAuditPathKind::WorthNativeDirectRead)
        .expect("read row should exist");
    let provenance = row
        .provenance()
        .support_pin()
        .expect("blocked row should still preserve pin provenance");

    assert!(!receipt.is_runtime_ready_for_phase_one());
    assert!(!receipt
        .rows_with_closure_posture(worth_server::WorthServerQueryDependencyClosurePosture::Blocked)
        .is_empty());
    assert!(receipt.support_posture().blocked_row_count() > 0);
    assert_eq!(
        row.consumer_kit_posture(),
        worth_server::WorthServerQueryDependencyConsumerKitPosture::QuerySupportSnapshotAndPinningBlocked
    );
    assert_eq!(provenance.workspace_name(), "audit-workspace");
    assert!(provenance.blocking_finding_count() > 0);
}
