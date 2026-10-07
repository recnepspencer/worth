use crate::runtime::tests::support::*;
use crate::runtime::worker_host::{
    certify_worker_phase7_performance_contracts, required_bridge_allocation_posture,
    required_complexity_contracts, required_counter_names, required_failure_modes,
    WorkerPhase7PerformanceContractPackage,
};

#[test]
fn worker_phase7_performance_contracts_certify_required_counter_and_cost_catalogue() {
    let package = certify_worker_phase7_performance_contracts().unwrap();

    assert_eq!(
        package.certification_family,
        "workerPhase7PerformanceContractCertification"
    );
    assert_eq!(package.covered_counter_count, 27);
    assert_eq!(package.covered_complexity_contract_count, 14);
    assert_eq!(package.prohibited_failure_mode_count, 8);
    assert!(package
        .counter_names
        .contains(&"workerTransactionSubmissionCount"));
    assert!(package
        .counter_names
        .contains(&"bridgeSerializationAllocationCount"));
    assert!(package.complexity_contracts.iter().any(|contract| {
        contract.operation == "diagnosticsSummaryReads"
            && contract.cost_bases.contains(&"zeroRichReconstruction")
    }));
    assert!(package
        .prohibited_failure_modes
        .iter()
        .any(|mode| mode.mode == "BridgeChatterStorm"));
    assert_eq!(
        package.bridge_allocation_posture.posture,
        "explicitBoundaryAllocationAccounting"
    );
    assert!(!package.bridge_allocation_posture.hidden_allocation_allowed);
    assert_digest_shape(&package.counter_catalog_digest);
    assert_digest_shape(&package.complexity_contract_digest);
    assert_digest_shape(&package.failure_mode_digest);
    assert_digest_shape(&package.bridge_allocation_posture_digest);
    assert_digest_shape(&package.certification_digest);
}

#[test]
fn worker_phase7_performance_contracts_reject_missing_required_counter() {
    let mut counters = required_counter_names();
    counters.retain(|counter| *counter != "workerFallbackCount");

    let error = WorkerPhase7PerformanceContractPackage::from_catalog(
        counters,
        required_complexity_contracts(),
        required_failure_modes(),
        required_bridge_allocation_posture(),
    )
    .unwrap_err();

    assert!(error.message.contains("workerFallbackCount"));
}

#[test]
fn worker_phase7_performance_contracts_reject_total_graph_cost_basis() {
    let counters = required_counter_names();
    let mut contracts = required_complexity_contracts();
    let output_delivery = contracts
        .iter_mut()
        .find(|contract| contract.operation == "committedOutputDelivery")
        .unwrap();
    output_delivery.cost_bases.push("totalGraphSize");

    let error = WorkerPhase7PerformanceContractPackage::from_catalog(
        counters,
        contracts,
        required_failure_modes(),
        required_bridge_allocation_posture(),
    )
    .unwrap_err();

    assert!(error.message.contains("forbidden cost base"));
}

#[test]
fn worker_phase7_performance_contracts_reject_missing_required_cost_basis() {
    let counters = required_counter_names();
    let mut contracts = required_complexity_contracts();
    let diagnostics_summary = contracts
        .iter_mut()
        .find(|contract| contract.operation == "diagnosticsSummaryReads")
        .unwrap();
    diagnostics_summary
        .cost_bases
        .retain(|basis| *basis != "zeroRichReconstruction");

    let error = WorkerPhase7PerformanceContractPackage::from_catalog(
        counters,
        contracts,
        required_failure_modes(),
        required_bridge_allocation_posture(),
    )
    .unwrap_err();

    assert!(error.message.contains("zeroRichReconstruction"));
}

#[test]
fn worker_phase7_performance_contracts_reject_hidden_bridge_allocation() {
    let mut posture = required_bridge_allocation_posture();
    posture.hidden_allocation_allowed = true;

    let error = WorkerPhase7PerformanceContractPackage::from_catalog(
        required_counter_names(),
        required_complexity_contracts(),
        required_failure_modes(),
        posture,
    )
    .unwrap_err();

    assert!(error.message.contains("bridge allocation posture"));
}

#[test]
fn worker_phase7_performance_contracts_reject_duplicate_coverage_entries() {
    let mut counters = required_counter_names();
    counters.push("workerFallbackCount");

    let error = WorkerPhase7PerformanceContractPackage::from_catalog(
        counters,
        required_complexity_contracts(),
        required_failure_modes(),
        required_bridge_allocation_posture(),
    )
    .unwrap_err();

    assert!(error.message.contains("duplicate counter"));
}
