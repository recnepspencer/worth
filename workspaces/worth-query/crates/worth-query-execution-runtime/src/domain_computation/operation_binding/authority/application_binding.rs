use std::sync::Arc;

use worth_query_admission::facade::basis::{
    AdmittedBasisCapability, MutationPreparationLaneWitness,
};
use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;
use worth_query_installation::facade::{
    WorthQueryCompiledApplicationOperationContracts, WorthQueryInstalledGraphParticipationAuthority,
};

use super::topology::resource_topology;
use super::WorthQueryExecutionBoundOperationAuthority;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::operation_binding::{
    WorthQueryExecutionCommitPosture, WorthQueryInstalledDomainExecutionAuthority,
    WorthQueryInstalledOperationExecutionSupport,
};
use crate::domain_computation::provider_session::{
    WorthQueryApplicationEffectPosture, WorthQueryProviderPlanDeclarations,
};

pub struct WorthQueryApplicationOperationBindingInput<'a> {
    pub runtime: &'a WorthQueryExecutionRuntime,
    pub owner: &'a str,
    pub installed_operation_fingerprint: Arc<str>,
    pub operation_slot: Arc<str>,
    pub resource_binding_identity: Arc<str>,
    pub basis: &'a AdmittedBasisCapability<MutationPreparationLaneWitness>,
    pub contracts: &'a WorthQueryCompiledApplicationOperationContracts,
    pub graph: &'a WorthQueryInstalledGraphParticipationAuthority,
    pub support: WorthQueryExecutionResourceSupportSnapshot,
    pub graph_work_session:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub graph_work_managed_run:
        crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity,
    pub operation_attempt:
        crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    pub schema_binding: &'a worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    pub product: &'a crate::basis::WorthQueryProductBranchLease,
    pub effect_posture: WorthQueryApplicationEffectPosture,
    pub native_reference_invariants:
        &'a worth_query_installation::facade::WorthQueryInvariantExecutionContract,
}

impl WorthQueryExecutionBoundOperationAuthority {
    pub fn bind_application(
        input: WorthQueryApplicationOperationBindingInput<'_>,
    ) -> Result<Self, &'static str> {
        let commit_posture = WorthQueryExecutionCommitPosture::Atomic;
        let topology = resource_topology(
            std::iter::empty(),
            &[input.graph],
            std::iter::once((
                "primary",
                worth_query_installation::facade::WorthQueryOperationGraphAccess::Project,
            )),
            std::iter::once("primary"),
            commit_posture,
        );
        Ok(Self {
            runtime_authority: input.runtime.authority_identity(),
            installation_runtime_ordinal: input.runtime.installed_packages().runtime_ordinal(),
            binding_identity: Arc::clone(&input.resource_binding_identity),
            operation_identity: Arc::clone(&input.installed_operation_fingerprint),
            basis_identity: input.basis.capability_digest().into(),
            semantic_basis: input.basis.normalized().clone(),
            canonical_query_digest: input.installed_operation_fingerprint,
            operation_resource_contract_identity: input
                .contracts
                .resources()
                .canonical_identity()
                .into(),
            provider_plan_declarations: Arc::new(
                WorthQueryProviderPlanDeclarations::from_application_contracts(
                    input.contracts,
                    input.effect_posture,
                    input.native_reference_invariants,
                )?,
            ),
            commit_posture,
            direct_resource_topology: topology,
            workflow_stage_resources: None,
            operation_evidence_contract: None,
            installed_support: WorthQueryInstalledOperationExecutionSupport::direct(input.support),
            installed_domain: WorthQueryInstalledDomainExecutionAuthority::mint(
                input.runtime.authority_identity(),
                input.owner,
                input.runtime.installed_packages().generation(),
                input.runtime.retain_current_generation(),
            ),
            graph_work_affinity: Some(super::WorthQueryApplicationGraphWorkAffinity {
                session: input.graph_work_session,
                managed_run: input.graph_work_managed_run,
            }),
            application_operation_attempt: Some(input.operation_attempt),
            application_operation_slot: Some(input.operation_slot),
            application_schema_binding: Some(input.schema_binding.clone()),
            application_snapshot: Some(input.snapshot.clone()),
            application_product_observation: Some(input.product.observation().clone()),
        })
    }
}
