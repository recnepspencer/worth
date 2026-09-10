use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_admission::facade::resource_admission::{
    WorthQueryAdmittedExecutionResourcePlan, WorthQueryAdmittedWorkflowResourcePlan,
};
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use crate::domain_computation::artifact_owner::WorthQueryInstalledWorkflowArtifactContracts;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionRuntime, WorthQueryRuntimeAuthorityIdentity,
};
use crate::domain_computation::operation_binding::{
    WorthQueryExecutionCommitPosture, WorthQueryInstalledDomainExecutionAuthority,
};
use crate::domain_computation::provider_session::WorthQueryGraphProviderCallKind;
use crate::domain_computation::WorthQueryInstalledOperationExecutionSupport;

mod application_binding;
mod provider_plan;
mod runtime_binding;
mod topology;

pub(crate) use application_binding::WorthQueryApplicationOperationBindingInput;
use topology::operation_workflow_topology;
use topology::WorthQueryExecutionResourceTopology;

#[derive(Clone)]
pub struct WorthQueryExecutionBoundOperationAuthority {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    installation_runtime_ordinal: u64,
    binding_identity: Arc<str>,
    operation_identity: Arc<str>,
    basis_identity: Arc<str>,
    semantic_basis: worth_query_admission::facade::basis::NormalizedBasisIntent,
    canonical_query_digest: Arc<str>,
    operation_resource_contract_identity: Arc<str>,
    provider_plan_declarations:
        Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations>,
    commit_posture: WorthQueryExecutionCommitPosture,
    direct_resource_topology: WorthQueryExecutionResourceTopology,
    workflow_stage_resources: Option<BTreeMap<Arc<str>, WorthQueryWorkflowStageResourceAuthority>>,
    operation_evidence_contract:
        Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>,
    installed_support: WorthQueryInstalledOperationExecutionSupport,
    installed_domain: Arc<WorthQueryInstalledDomainExecutionAuthority>,
    graph_work_affinity: Option<WorthQueryApplicationGraphWorkAffinity>,
    application_operation_attempt:
        Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>,
    application_operation_slot: Option<Arc<str>>,
    application_schema_binding:
        Option<worth_query_installation::facade::ApplicationSchemaBindingIdentity>,
    application_snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    application_product_observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

#[derive(Clone, Copy)]
pub(crate) struct WorthQueryApplicationGraphWorkAffinity {
    pub(crate) session:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub(crate) managed_run:
        crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity,
}

impl std::fmt::Debug for WorthQueryExecutionBoundOperationAuthority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryExecutionBoundOperationAuthority")
            .field("binding_identity", &self.binding_identity)
            .field("operation_identity", &self.operation_identity)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct WorthQueryWorkflowStageResourceAuthority {
    contract_identity: Arc<str>,
    topology: WorthQueryExecutionResourceTopology,
    predecessors: Arc<[String]>,
    artifact_contracts: WorthQueryInstalledWorkflowArtifactContracts,
}

impl WorthQueryExecutionBoundOperationAuthority {
    pub(crate) const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.runtime_authority
    }

    pub(crate) const fn graph_work_affinity(
        &self,
    ) -> Option<WorthQueryApplicationGraphWorkAffinity> {
        self.graph_work_affinity
    }

    pub(crate) const fn application_operation_attempt(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>
    {
        self.application_operation_attempt
    }

    pub(crate) const fn application_schema_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        self.application_schema_binding.as_ref()
    }

    pub(crate) const fn application_operation_slot(&self) -> Option<&Arc<str>> {
        self.application_operation_slot.as_ref()
    }

    pub(crate) const fn application_snapshot(
        &self,
    ) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        self.application_snapshot.as_ref()
    }

    pub(crate) const fn application_product_observation(
        &self,
    ) -> Option<&worth_runtime_world::facade::ProductBranchObservation> {
        self.application_product_observation.as_ref()
    }

    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }

    pub fn basis_identity(&self) -> &str {
        &self.basis_identity
    }

    pub fn semantic_basis(&self) -> &worth_query_admission::facade::basis::NormalizedBasisIntent {
        &self.semantic_basis
    }

    pub fn commit_posture(&self) -> WorthQueryExecutionCommitPosture {
        self.commit_posture
    }

    pub(crate) fn retain_installed_domain_authority(
        &self,
    ) -> Arc<WorthQueryInstalledDomainExecutionAuthority> {
        Arc::clone(&self.installed_domain)
    }

    pub fn direct_support(
        &self,
    ) -> Option<&worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot>
    {
        self.installed_support.direct_operation()
    }

    pub fn workflow_operation_support(
        &self,
    ) -> Option<&worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot>
    {
        self.installed_support.workflow_operation()
    }

    pub fn workflow_stage_support(
        &self,
        stage_identity: &str,
    ) -> Option<&worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot>
    {
        self.installed_support.workflow_stage(stage_identity)
    }

    pub(crate) fn belongs_to(&self, runtime: &WorthQueryExecutionRuntime) -> bool {
        self.runtime_authority == runtime.authority_identity()
    }

    pub(crate) fn belongs_to_current_installation(
        &self,
        runtime: &WorthQueryExecutionRuntime,
    ) -> bool {
        self.belongs_to(runtime) && self.installed_domain.is_current_installation_generation()
    }

    pub(crate) fn admits_convergence_contract(
        &self,
        contract: &worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract,
    ) -> bool {
        self.installation_runtime_ordinal == contract.runtime_ordinal()
            && self.operation_identity() == contract.operation_identity()
            && self.installed_domain.owner() == contract.operation_owner()
            && self.installation_generation() == contract.generation()
            && self.operation_resource_contract_identity.as_ref()
                == contract.resource_contract_identity()
            && self
                .convergence_evidence_contract(contract)
                .is_some_and(|installed| {
                    installed.admission_identity() == contract.artifact_admission_identity()
                        && installed.contract().identity().as_str()
                            == contract.artifact_contract_identity()
                })
    }

    fn convergence_evidence_contract(
        &self,
        contract: &worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract,
    ) -> Option<&worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>
    {
        match contract.evidence_stage_identity() {
            None => self.operation_evidence_contract().map(AsRef::as_ref),
            Some(stage) => self
                .workflow_stage_artifact_contracts(stage)?
                .evidence()
                .map(AsRef::as_ref),
        }
    }

    pub fn installation_generation(
        &self,
    ) -> worth_query_installation::facade::WorthQueryInstallationGeneration {
        self.installed_domain.installation_generation()
    }

    pub fn is_current_installation_generation(&self) -> bool {
        self.installed_domain.is_current_installation_generation()
    }

    pub(crate) fn is_workflow_operation(&self) -> bool {
        self.workflow_stage_resources.is_some()
    }

    pub(crate) fn admits_direct_plan(
        &self,
        plan: &WorthQueryAdmittedExecutionResourcePlan,
    ) -> bool {
        self.workflow_stage_resources.is_none()
            && self.admits_operation_plan(plan)
            && self
                .installed_support
                .direct_operation()
                .is_some_and(|support| support == plan.support_snapshot())
            && self
                .direct_resource_topology
                .admits(plan.support_snapshot())
            && plan.support_snapshot().parallel_admission().is_none()
    }

    pub(crate) fn admits_workflow_plan(
        &self,
        plan: &WorthQueryAdmittedWorkflowResourcePlan,
    ) -> bool {
        let Some(expected_stages) = &self.workflow_stage_resources else {
            return false;
        };
        self.admits_operation_plan(plan.operation())
            && self
                .installed_support
                .workflow_operation()
                .is_some_and(|support| support == plan.operation().support_snapshot())
            && operation_workflow_topology(self).admits(plan.operation().support_snapshot())
            && plan.stages().count() == expected_stages.len()
            && plan.stages().all(|(stage_identity, stage_plan)| {
                expected_stages
                    .get(stage_identity)
                    .is_some_and(|authority| {
                        stage_plan.binding_identity()
                            == format!("{}:{stage_identity}", self.binding_identity)
                            && stage_plan.contract_identity()
                                == authority.contract_identity.as_ref()
                            && authority.topology.admits(stage_plan.support_snapshot())
                            && self
                                .installed_support
                                .workflow_stage(stage_identity)
                                .is_some_and(|support| support == stage_plan.support_snapshot())
                            && stage_plan.support_snapshot().parallel_admission().is_none()
                    })
            })
    }

    pub(crate) fn admits_convergence_graph(
        &self,
        contract: &worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract,
        graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    ) -> bool {
        match contract.evidence_stage_identity() {
            None => {
                self.workflow_stage_resources.is_none()
                    && self
                        .direct_resource_topology
                        .contains_graph_authority(graph)
            }
            Some(stage) => self
                .workflow_stage_resources
                .as_ref()
                .and_then(|stages| stages.get(stage))
                .is_some_and(|stage| stage.topology.contains_graph_authority(graph)),
        }
    }

    fn admits_operation_plan(&self, plan: &WorthQueryAdmittedExecutionResourcePlan) -> bool {
        plan.binding_identity() == self.binding_identity()
            && plan.contract_identity() == self.operation_resource_contract_identity.as_ref()
    }

    pub(crate) fn admits_graph_call(
        &self,
        stage_identity: Option<&str>,
        authority: &WorthQueryInstalledGraphParticipationAuthority,
        kind: WorthQueryGraphProviderCallKind,
    ) -> bool {
        if authority.runtime_ordinal() != self.installation_runtime_ordinal {
            return false;
        }
        match (stage_identity, &self.workflow_stage_resources) {
            (None, None) => self
                .direct_resource_topology
                .admits_graph_call(authority, kind),
            (Some(stage), Some(stages)) => stages
                .get(stage)
                .is_some_and(|stage| stage.topology.admits_graph_call(authority, kind)),
            _ => false,
        }
    }

    pub(crate) fn admits_commit_call(
        &self,
        stage_identity: Option<&str>,
        authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    ) -> bool {
        match (stage_identity, &self.workflow_stage_resources) {
            (None, None) => self
                .direct_resource_topology
                .admits_commit_call(authorities),
            (Some(stage), Some(stages)) => stages
                .get(stage)
                .is_some_and(|stage| stage.topology.admits_commit_call(authorities)),
            _ => false,
        }
    }

    pub(crate) fn canonical_query_digest(&self) -> &str {
        &self.canonical_query_digest
    }

    pub(crate) fn operation_evidence_contract(
        &self,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.operation_evidence_contract.as_ref()
    }

    /// Descriptive contract content for high-level ordinary direct completion.
    /// This does not mint or attach execution evidence.
    pub fn ordinary_direct_domain_evidence_contract(
        &self,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.operation_evidence_contract.as_ref()
    }

    /// Descriptive contract content for a high-level validated workflow stage.
    /// This does not mint or attach execution evidence.
    pub fn ordinary_workflow_domain_evidence_contract(
        &self,
        stage_identity: &str,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.workflow_stage_artifact_contracts(stage_identity)?
            .evidence()
    }

    pub(crate) fn workflow_stage_artifact_contracts(
        &self,
        stage_identity: &str,
    ) -> Option<&WorthQueryInstalledWorkflowArtifactContracts> {
        self.workflow_stage_resources
            .as_ref()?
            .get(stage_identity)
            .map(|stage| &stage.artifact_contracts)
    }

    pub(crate) fn admits_workflow_edge(
        &self,
        predecessor_stage: &str,
        consumer_stage: &str,
    ) -> bool {
        self.workflow_stage_resources
            .as_ref()
            .and_then(|stages| stages.get(consumer_stage))
            .is_some_and(|stage| {
                stage
                    .predecessors
                    .iter()
                    .any(|predecessor| predecessor == predecessor_stage)
            })
    }
}

#[cfg(test)]
pub(crate) mod tests;
