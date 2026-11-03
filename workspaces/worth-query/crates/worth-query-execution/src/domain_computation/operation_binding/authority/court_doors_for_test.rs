//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn artifact_contracts_for_test(&self) -> &WorthQueryInstalledWorkflowArtifactContracts {
        &self.artifact_contracts
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn direct_resource_topology_for_test(&self) -> &WorthQueryExecutionResourceTopology {
        &self.direct_resource_topology
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn installation_runtime_ordinal_for_test(&self) -> &u64 {
        &self.installation_runtime_ordinal
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn provider_plan_declarations_for_test(&self) -> &Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations> {
        &self.provider_plan_declarations
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn topology_for_test(&self) -> &WorthQueryExecutionResourceTopology {
        &self.topology
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn artifact_contracts_mut_for_test(&mut self) -> &mut WorthQueryInstalledWorkflowArtifactContracts {
        &mut self.artifact_contracts
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn direct_resource_topology_mut_for_test(&mut self) -> &mut WorthQueryExecutionResourceTopology {
        &mut self.direct_resource_topology
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn installation_runtime_ordinal_mut_for_test(&mut self) -> &mut u64 {
        &mut self.installation_runtime_ordinal
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn provider_plan_declarations_mut_for_test(&mut self) -> &mut Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations> {
        &mut self.provider_plan_declarations
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn topology_mut_for_test(&mut self) -> &mut WorthQueryExecutionResourceTopology {
        &mut self.topology
    }
}

impl WorthQueryExecutionBoundOperationAuthority {
    #[doc(hidden)]
    pub fn new_for_test(runtime_authority: WorthQueryRuntimeAuthorityIdentity, installation_runtime_ordinal: u64, binding_identity: Arc<str>, operation_identity: Arc<str>, basis_identity: Arc<str>, semantic_basis: worth_query_admission::facade::basis::NormalizedBasisIntent, canonical_query_digest: Arc<str>, operation_resource_contract_identity: Arc<str>, provider_plan_declarations: Arc<crate::domain_computation::provider_session::WorthQueryProviderPlanDeclarations>, commit_posture: WorthQueryExecutionCommitPosture, direct_resource_topology: WorthQueryExecutionResourceTopology, workflow_stage_resources: Option<BTreeMap<Arc<str>, WorthQueryWorkflowStageResourceAuthority>>, operation_evidence_contract: Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>, installed_support: WorthQueryInstalledOperationExecutionSupport, installed_domain: Arc<WorthQueryInstalledDomainExecutionAuthority>, graph_work_affinity: Option<WorthQueryApplicationGraphWorkAffinity>, application_operation_attempt: Option<crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity>, application_operation_slot: Option<Arc<str>>, application_schema_binding: Option<worth_query_installation::facade::ApplicationSchemaBindingIdentity>, application_snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>, application_product_observation: Option<worth_runtime_world::facade::ProductBranchObservation>) -> Self {
        Self { runtime_authority, installation_runtime_ordinal, binding_identity, operation_identity, basis_identity, semantic_basis, canonical_query_digest, operation_resource_contract_identity, provider_plan_declarations, commit_posture, direct_resource_topology, workflow_stage_resources, operation_evidence_contract, installed_support, installed_domain, graph_work_affinity, application_operation_attempt, application_operation_slot, application_schema_binding, application_snapshot, application_product_observation }
    }
}

impl WorthQueryWorkflowStageResourceAuthority {
    #[doc(hidden)]
    pub fn new_for_test(contract_identity: Arc<str>, topology: WorthQueryExecutionResourceTopology, predecessors: Arc<[String]>, artifact_contracts: WorthQueryInstalledWorkflowArtifactContracts) -> Self {
        Self { contract_identity, topology, predecessors, artifact_contracts }
    }
}
