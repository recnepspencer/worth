use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_contract_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_generation_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_runtime_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::assert_standard_mismatch;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::artifact_owner::authority_affinity_tests::installed_contracts;

use std::sync::Arc;
use worth_foundational::facade::CanonicalizationRuleVersion;
use worth_foundational::facade::FoundationalPerformanceCounterName;
use worth_foundational::facade::RetentionDeliveryProfile;
use worth_query_execution::court_doors_for_test::domain_computation::artifact_owner::production_authority::WorthQueryArtifactProductionAuthorityParts;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionGeneration;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;
use worth_query_execution::facade::domain_computation::WorthQueryInstalledDomainExecutionAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistry;
use worth_query_installation::facade::WorthQueryArtifactClassification;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityContract;
use worth_query_installation::facade::WorthQueryArtifactCompatibilityWindow;
use worth_query_installation::facade::WorthQueryArtifactContentIdentityContract;
use worth_query_installation::facade::WorthQueryArtifactDeletionPosture;
use worth_query_installation::facade::WorthQueryArtifactDeterminismPosture;
use worth_query_installation::facade::WorthQueryArtifactDowngradePosture;
use worth_query_installation::facade::WorthQueryArtifactEvidenceContract;
use worth_query_installation::facade::WorthQueryArtifactFamily;
use worth_query_installation::facade::WorthQueryArtifactLegalHoldPosture;
use worth_query_installation::facade::WorthQueryArtifactLifecycleContract;
use worth_query_installation::facade::WorthQueryArtifactOccurrenceContract;
use worth_query_installation::facade::WorthQueryArtifactOwnershipContract;
use worth_query_installation::facade::WorthQueryArtifactProtocolVersion;
use worth_query_installation::facade::WorthQueryArtifactRedactionPosture;
use worth_query_installation::facade::WorthQueryArtifactReproducibilityClass;
use worth_query_installation::facade::WorthQueryArtifactReproducibilityContract;
use worth_query_installation::facade::WorthQueryArtifactRetirementRule;
use worth_query_installation::facade::WorthQueryArtifactSchemaVersion;
use worth_query_installation::facade::WorthQueryArtifactVersionSupport;
use worth_query_installation::facade::WorthQueryCandidateSearchContract;
use worth_query_installation::facade::WorthQueryConvergenceContract;
use worth_query_installation::facade::WorthQueryDecisionRecordContract;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryInstallationRuntimeIdentity;
use worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority;
use worth_query_installation::facade::WorthQueryInstalledPackageIndex;
use worth_query_installation::facade::WorthQueryPortableArtifactContract;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;
use worth_query_installation::facade::WorthQueryStructuralCounterContract;
use worth_query_installation::facade::WorthQueryTransformationEvidenceContract;
#[test]
fn every_affinity_dimension_uses_real_installed_authority_and_exact_denial() {
    let contracts = installed_contracts();
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::OperationMismatch,
        |parts| parts.operation_identity = "foreign-operation".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::OperationMismatch,
        |parts| parts.binding_identity = "foreign-binding".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::RunMismatch,
        |parts| {
            parts.run_identity = "foreign-run".into();
        },
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::StageMismatch,
        |parts| parts.stage_identity = "foreign-stage".into(),
    );
    assert_standard_mismatch(
        &contracts,
        WorthQueryArtifactDenialKind::BasisMismatch,
        |parts| parts.basis_identity = "foreign-basis".into(),
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
