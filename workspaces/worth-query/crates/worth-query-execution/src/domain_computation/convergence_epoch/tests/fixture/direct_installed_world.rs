use std::sync::Arc;

use worth_query_admission::facade::convergence_epoch::WorthQueryAdmittedConvergenceContract;
use worth_query_admission::facade::resource_admission::{
    WorthQueryExecutionResourceAdmissionCounters, WorthQueryExecutionResourceSupportSnapshot,
};
use worth_query_admission::integration::admit_execution_resource_plan;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_installation::facade::{
    WorthQueryArtifactFamily, WorthQueryInstallationGeneration,
    WorthQueryInstalledGraphParticipationAuthority,
};

use crate::domain_computation::managed_run::tests::causal_fixture;
use crate::domain_computation::operation_binding::{
    WorthQueryExecutionCommitPosture, WorthQueryInstalledOperationExecutionSupport,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use crate::domain_computation::{
    WorthQueryAdmittedDirectRun, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
    WorthQueryIteratingDirectConvergenceEpoch,
};

use super::admitted_basis::{admitted_alternate_basis, admitted_basis};
use super::candidate_contract::FixtureConvergenceContract;
use super::fixture_identity::{CandidateFamily, GRAPH_ROLE, OPERATION_SLOT, OWNER};
use super::package::admitted_package_with_contract;
use super::provider::{
    ConvergentProvider, FixtureDisposition, FixtureDomainPortProbe, FixtureGraph,
    FixtureReportHistoryProbe, FixtureYieldRecoveryProbe,
};
use super::resource_contract::resource_contract;

pub(crate) fn direct_epoch_fixture(
    disposition: FixtureDisposition,
) -> WorthQueryIteratingDirectConvergenceEpoch {
    direct_admission_fixture(disposition).admit()
}

pub(crate) struct DirectAdmissionFixture {
    pub runtime: WorthQueryExecutionRuntime,
    pub operation: WorthQueryExecutionBoundOperationAuthority,
    pub alternate_basis_operation: WorthQueryExecutionBoundOperationAuthority,
    pub contract: WorthQueryAdmittedConvergenceContract,
    pub managed: WorthQueryAdmittedDirectRun,
    pub graph: WorthQueryInstalledGraphParticipationAuthority,
    pub bridge: worth_runtime_bridge::facade::RuntimeBridge,
}

impl DirectAdmissionFixture {
    pub(crate) fn admit(self) -> WorthQueryIteratingDirectConvergenceEpoch {
        match self.runtime.admit_direct_convergence_epoch(
            &self.operation,
            self.contract,
            self.managed,
            self.graph,
        ) {
            Ok(epoch) => epoch.start(),
            Err(_) => panic!("exact installed authorities must admit convergence epoch"),
        }
    }
}

pub(crate) fn direct_admission_fixture(disposition: FixtureDisposition) -> DirectAdmissionFixture {
    direct_admission_fixture_with_contract(disposition, FixtureConvergenceContract::Bounded)
}

pub(crate) fn direct_admission_fixture_with_domain_port_probe(
    disposition: FixtureDisposition,
) -> (DirectAdmissionFixture, FixtureDomainPortProbe) {
    let probe = FixtureDomainPortProbe::default();
    let provider = ConvergentProvider::new(disposition).with_domain_port_probe(probe.clone());
    (
        direct_admission_fixture_with_provider(FixtureConvergenceContract::Bounded, provider),
        probe,
    )
}

pub(crate) fn direct_admission_fixture_with_report_history_probe(
    disposition: FixtureDisposition,
) -> (DirectAdmissionFixture, FixtureReportHistoryProbe) {
    direct_admission_fixture_with_contract_and_report_history_probe(
        disposition,
        FixtureConvergenceContract::Bounded,
    )
}

pub(crate) fn direct_admission_fixture_with_contract_and_report_history_probe(
    disposition: FixtureDisposition,
    convergence_contract: FixtureConvergenceContract,
) -> (DirectAdmissionFixture, FixtureReportHistoryProbe) {
    let probe = FixtureReportHistoryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_report_history_probe(probe.clone());
    (
        direct_admission_fixture_with_provider(convergence_contract, provider),
        probe,
    )
}

pub(crate) fn direct_admission_fixture_with_contract(
    disposition: FixtureDisposition,
    convergence_contract: FixtureConvergenceContract,
) -> DirectAdmissionFixture {
    direct_admission_fixture_with_provider(
        convergence_contract,
        ConvergentProvider::new(disposition),
    )
}

pub(crate) fn direct_yield_recovery_admission_fixture(
) -> (DirectAdmissionFixture, FixtureYieldRecoveryProbe) {
    let disposition = FixtureDisposition::YieldThenSuspensionFailure;
    let probe = FixtureYieldRecoveryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_yield_recovery_probe(probe.clone());
    (
        direct_admission_fixture_with_provider(FixtureConvergenceContract::Bounded, provider),
        probe,
    )
}

pub(crate) fn direct_yield_denial_admission_fixture(
) -> (DirectAdmissionFixture, FixtureYieldRecoveryProbe) {
    let disposition = FixtureDisposition::YieldThenCheckpointUnavailable;
    let probe = FixtureYieldRecoveryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_yield_recovery_probe(probe.clone());
    (
        direct_admission_fixture_with_provider(FixtureConvergenceContract::Bounded, provider),
        probe,
    )
}

fn direct_admission_fixture_with_provider(
    convergence_contract: FixtureConvergenceContract,
    provider: ConvergentProvider,
) -> DirectAdmissionFixture {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(WorthQueryGraphProviderAnchor::install_convergent::<
        FixtureGraph,
        _,
    >(provider));
    let graph_support = anchor.resource_support().clone();
    let resources = resource_contract(&graph_support);
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        GRAPH_ROLE,
        "convergence-provider",
        false,
        Option::<String>::None,
        anchor,
    )
    .expect("convergence graph authority must install");
    let (runtime, installation_authority) = installer
        .install(
            WorthQueryInstallationGeneration::initial(),
            [admitted_package_with_contract(
                resources,
                convergence_contract,
            )],
        )
        .expect("convergence Query runtime must install")
        .into_parts();
    let operation = runtime
        .installed_packages()
        .domain_operation(OWNER, OPERATION_SLOT)
        .expect("fixture operation must be installed");
    let artifact = runtime
        .installed_packages()
        .artifact_contract(
            OWNER,
            CandidateFamily::SEMANTIC_FAMILY,
            worth_query_installation::facade::WorthQueryArtifactSchemaVersion::new(1),
            worth_query_installation::facade::WorthQueryArtifactProtocolVersion::new(1),
        )
        .expect("fixture convergence artifact must be installed");
    let convergence =
        worth_query_admission::facade::convergence_epoch::admit_convergence_epoch_contract(
            &operation, artifact,
        )
        .expect("installed operation and artifact must admit convergence");
    let support = WorthQueryExecutionResourceSupportSnapshot::new(
        graph_support.clone(),
        Vec::new(),
        vec![(GRAPH_ROLE.to_owned(), graph_support)],
        Vec::new(),
        None,
    );
    let basis = admitted_basis();
    let bound = runtime
        .bind_standalone_test_domain_operation(
            &installation_authority,
            &operation,
            &basis,
            &[&graph],
            &[],
            WorthQueryExecutionCommitPosture::ReadOnly,
            WorthQueryInstalledOperationExecutionSupport::direct(support.clone()),
        )
        .expect("real installed operation must bind to exact graph authority");
    let alternate_basis_operation = runtime
        .bind_standalone_test_domain_operation(
            &installation_authority,
            &operation,
            &admitted_alternate_basis(),
            &[&graph],
            &[],
            WorthQueryExecutionCommitPosture::ReadOnly,
            WorthQueryInstalledOperationExecutionSupport::direct(support.clone()),
        )
        .expect("same installed operation must bind to the alternate fixture basis");
    let executor_envelope = bound
        .direct_support()
        .expect("direct support must exist")
        .executor()
        .envelope();
    let request = WorthQueryExecutionResourceRequest::bounded(
        8,
        8,
        executor_envelope.cancellation_safe_point().clone(),
    )
    .allow_yielded_state_posture(executor_envelope.yielded_state_posture())
    .allow_retained_progress_posture(executor_envelope.retained_progress_posture());
    let plan = admit_execution_resource_plan(
        bound.binding_identity(),
        &operation.definition().semantics().resources,
        &request,
        support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .expect("fixture resource plan must admit");
    let attempt = runtime
        .start_direct_resource_attempt(&bound, plan)
        .expect("fixture resource attempt must start");
    let lower = causal_fixture::managed_admission_context();
    let managed = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_direct(&bound, attempt, lower.read_request())
        .expect("fixture managed run must admit through Bridge and Relational authorities");
    DirectAdmissionFixture {
        runtime,
        operation: bound,
        alternate_basis_operation,
        contract: convergence,
        managed,
        graph,
        bridge: lower.bridge,
    }
}
