use worth_query_installation::facade::WorthQueryArtifactFamily;
use worth_query_installation::facade::WorthQueryInstallationGeneration;
use worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority;
use worth_query_installation::facade::WorthQueryInstalledDomainOperationAuthority;

use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntime;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionRuntimeInstaller;

use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::FixtureConvergenceContract;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::CandidateFamily;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::OPERATION_SLOT;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::OWNER;
use crate::domain_computation::convergence_epoch::tests::fixture::package::admitted_package_with_contract;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::resource_support::execution_support;
use crate::domain_computation::convergence_epoch::tests::fixture::resource_contract::resource_contract;

pub struct StaticConvergenceAdmissionFixture {
    pub operation: WorthQueryInstalledDomainOperationAuthority,
    pub artifact: WorthQueryInstalledArtifactContractAuthority,
    pub _runtime: WorthQueryExecutionRuntime,
}

pub fn static_convergence_admission_fixture(
    convergence: FixtureConvergenceContract,
) -> StaticConvergenceAdmissionFixture {
    let resources = resource_contract(&execution_support(false));
    let (runtime, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            [admitted_package_with_contract(resources, convergence)],
        )
        .expect("static convergence fixture package must install")
        .into_parts();
    let operation = runtime
        .installed_packages()
        .domain_operation(OWNER, OPERATION_SLOT)
        .expect("static fixture operation must be installed");
    let artifact = runtime
        .installed_packages()
        .artifact_contract(
            OWNER,
            CandidateFamily::SEMANTIC_FAMILY,
            worth_query_installation::facade::WorthQueryArtifactSchemaVersion::new(1),
            worth_query_installation::facade::WorthQueryArtifactProtocolVersion::new(1),
        )
        .expect("static fixture artifact must be installed");
    StaticConvergenceAdmissionFixture {
        operation,
        artifact,
        _runtime: runtime,
    }
}
