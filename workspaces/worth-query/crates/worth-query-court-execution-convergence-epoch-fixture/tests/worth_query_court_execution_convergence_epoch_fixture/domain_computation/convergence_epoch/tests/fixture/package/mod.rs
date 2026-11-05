use worth_query_installation::facade::WorthQueryAdmittedPortableDomainPackage;
use worth_query_installation::facade::WorthQueryArtifactProtocolVersion;
use worth_query_installation::facade::WorthQueryArtifactSchemaVersion;
use worth_query_installation::facade::WorthQueryArtifactVersionSupport;
use worth_query_installation::facade::WorthQueryExecutionResourceContract;
use worth_query_installation::facade::WorthQueryInstallationAdmissionProfile;
use worth_query_installation::facade::WorthQueryInstallationSupportStatus;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_query_installation::facade::WorthQueryPortableDomainIdentity;
use worth_query_installation::facade::WorthQueryPortableDomainPackage;
use worth_query_installation::facade::WorthQueryValidatedPortableDomainPackage;

use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::candidate_contract;
use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::FixtureConvergenceContract;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::CandidateFamily;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::ComparatorFamily;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::OPERATION_SLOT;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::OWNER;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::operation_definition::direct_operation;
use crate::domain_computation::convergence_epoch::tests::fixture::operation_definition::workflow_operation;

pub fn admitted_package_with_contract(
    resources: WorthQueryExecutionResourceContract,
    convergence: FixtureConvergenceContract,
) -> WorthQueryAdmittedPortableDomainPackage {
    let contract = candidate_contract(OPERATION_SLOT, convergence);
    let reference = contract.reference();
    let package =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(OWNER, 1, 0))
            .artifact_contract(contract)
            .domain_operation(direct_operation(
                reference,
                resources,
                WorthQueryOperationGraphAccess::Observe,
            ))
            .validate()
            .expect("convergence fixture package must validate");
    admit_package(package)
}

pub fn admitted_workflow_package(
    operation_resources: WorthQueryExecutionResourceContract,
    stage_resources: WorthQueryExecutionResourceContract,
    graph_access: WorthQueryOperationGraphAccess,
) -> WorthQueryAdmittedPortableDomainPackage {
    let contract = candidate_contract(WORKFLOW_STAGE, FixtureConvergenceContract::Bounded);
    let reference = contract.reference();
    let package =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(OWNER, 1, 0))
            .artifact_contract(contract)
            .domain_operation(workflow_operation(
                reference,
                operation_resources,
                stage_resources,
                graph_access,
            ))
            .validate()
            .expect("workflow convergence fixture package must validate");
    admit_package(package)
}

pub fn admit_package(
    package: WorthQueryValidatedPortableDomainPackage,
) -> WorthQueryAdmittedPortableDomainPackage {
    WorthQueryInstallationAdmissionProfile::new("convergence-support-v1", "convergence-config-v1")
        .artifact_version::<CandidateFamily>(
            WorthQueryArtifactSchemaVersion::new(1),
            WorthQueryArtifactProtocolVersion::new(1),
            WorthQueryArtifactVersionSupport::Admitted,
        )
        .artifact_comparator::<ComparatorFamily>(WorthQueryInstallationSupportStatus::Admitted)
        .admit(package)
        .expect("convergence fixture package must admit")
}
