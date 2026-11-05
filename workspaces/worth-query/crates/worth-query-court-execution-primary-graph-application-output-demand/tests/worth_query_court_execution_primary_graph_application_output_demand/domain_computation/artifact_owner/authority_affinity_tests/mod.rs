use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_contract_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_generation_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_runtime_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_standard_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_contracts;

use std::sync::Arc;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactDenialKind;
#[test]
fn every_affinity_dimension_uses_real_installed_authority_and_exact_denial() {
    let contracts = installed_contracts();
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::OperationMismatch,
        |parts| (*parts.operation_identity_mut_for_test()) = "foreign-operation".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::OperationMismatch,
        |parts| (*parts.binding_identity_mut_for_test()) = "foreign-binding".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::RunMismatch,
        |parts| {
            (*parts.run_identity_mut_for_test()) = "foreign-run".into();
        },
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::StageMismatch,
        |parts| (*parts.stage_identity_mut_for_test()) = "foreign-stage".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::BasisMismatch,
        |parts| (*parts.basis_identity_mut_for_test()) = "foreign-basis".into(),
    );
    assert_contract_mismatch(
        &contracts,
        Arc::clone(&contracts.foreign_owner),
        WorthQueryArtifactDenialKind::PayloadOwnerMismatch,
    );
    assert_contract_mismatch(
        &contracts,
        Arc::clone(&contracts.foreign_family),
        WorthQueryArtifactDenialKind::ArtifactContractMismatch,
    );
    assert_contract_mismatch(
        &contracts,
        Arc::clone(&contracts.foreign_version),
        WorthQueryArtifactDenialKind::ArtifactContractMismatch,
    );
    assert_runtime_mismatch(&contracts);
    assert_generation_mismatch(&contracts);
}
