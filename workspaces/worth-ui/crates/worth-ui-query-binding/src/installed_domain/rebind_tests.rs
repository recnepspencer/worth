use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};

use super::{
    WorthUiQueryDomainRebindDenialKind, WorthUiQueryDomainRebindNextAction,
    WorthUiQueryWorkspaceExt,
};

#[test]
fn denial_preserves_query_owned_evidence_and_next_action() {
    let source = installed_measurement_workspace("worth-ui-rebind-source");
    let installed = source
        .worth_ui()
        .expect("Worth UI domain should be installed");
    let target = measurement_workspace_without_domain("worth-ui-rebind-missing-target");

    let denial = installed
        .rebind_to(&target)
        .expect_err("a target without the domain package must deny rebind");

    assert_eq!(
        denial.kind(),
        WorthUiQueryDomainRebindDenialKind::DomainNotInstalled
    );
    assert_eq!(
        denial.next_action(),
        WorthUiQueryDomainRebindNextAction::InstallDomainPackage
    );
    assert_eq!(
        denial.prior_package_identity(),
        installed.handle().package_identity()
    );
    assert!(denial.current_package_identity().is_none());
    assert_eq!(denial.counters().planning_attempts(), 0);
    assert_eq!(denial.counters().lower_runtime_attempts(), 0);
    assert_eq!(denial.counters().execution_attempts(), 0);
    let inspection = crate::WorthUiQueryInspection::exact_artifact(
        &denial,
        crate::WorthUiQueryInspectionRelevance::Relevant,
    );
    assert!(std::ptr::eq(inspection.exact_artifact(), &denial));
    assert_eq!(inspection.counters().rich_evidence_section_count(), 0);
}

#[test]
fn success_retains_query_semantic_equivalence_receipt() {
    let source = installed_measurement_workspace("worth-ui-rebind-receipt-source");
    let installed = source
        .worth_ui()
        .expect("Worth UI domain should be installed");
    let target = installed_measurement_workspace("worth-ui-rebind-receipt-target");

    let rebound = installed
        .rebind_to(&target)
        .expect("equivalent installed package meaning should rebind");

    assert_eq!(
        rebound.query_receipt().package_identity(),
        rebound.current().handle().package_identity()
    );
    assert_eq!(rebound.prior(), &installed);
    assert!(!rebound.prior().shares_authority_with(rebound.current()));
}

fn installed_measurement_workspace(
    name: &str,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        crate::install_worth_ui_test_operation_executors(
            in_memory_test_runtime()
                .with_schema(measurement_schema())
                .domain_package(crate::worth_ui_domain_package()),
        )
        .workspace(execution, name)
        .expect("Worth UI rebind workspace should build")
    })
}

fn measurement_workspace_without_domain(
    name: &str,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::certification::request_custody::with_bootstrap_request(|execution| {
        in_memory_test_runtime()
            .with_schema(measurement_schema())
            .workspace(execution, name)
            .expect("Query workspace without Worth UI should build")
    })
}

fn measurement_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("WorthUiMeasurement")
        .aspect_contracts(crate::worth_ui_native_aspect_contracts())
        .expect("Worth UI native aspect contracts should build")
        .aspect("identity.id", "identity.id")
        .expect("identity aspect should build")
        .aspect("measurement.value", "measurement.value")
        .expect("measurement aspect should build")
}
