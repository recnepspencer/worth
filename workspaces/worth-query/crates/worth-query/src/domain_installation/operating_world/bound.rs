mod correspondence;

use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::basis_lifecycle::{AdmittedBasisCapability, BasisOperationLane};
use crate::domain_installation::operation_authority_chain::{
    mint_operation_phase_proof, WorthQueryBoundOperationPhase, WorthQueryOperationAuthorityBasis,
    WorthQueryOperationPhaseProof,
};

use super::super::{
    WorthQueryConsumerProjectionContract, WorthQueryConsumerProjectionContractDenial,
    WorthQueryConsumerSupportProfile, WorthQueryExecutableDomainOperation,
    WorthQueryInstalledDomainOperation, WorthQueryInstalledDomainOperationExecutor,
    WorthQueryInstalledGraphParticipationRecord, WorthQueryPublishingOperation,
};
use super::execution_support::{
    WorthQueryBoundExecutionProviders, WorthQueryBoundWorkflowParallelPosture,
};

type BoundOperationMarker<D, O, F> = fn() -> (D, O, F);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBoundCommitPosture {
    ReadOnly,
    Atomic,
    Compensated,
}

pub(crate) struct WorthQueryBoundGraphParticipation {
    pub(crate) role: String,
    pub(crate) record: Arc<WorthQueryInstalledGraphParticipationRecord>,
}

pub(crate) struct WorthQueryBoundRequiredDomain {
    pub(crate) role: String,
    pub(crate) authority: Arc<super::super::WorthQueryInstalledDomainAuthority>,
}

pub(crate) struct WorthQueryBoundAuthoritySet {
    pub(crate) graph_participations: Vec<WorthQueryBoundGraphParticipation>,
    pub(crate) required_domains: Vec<WorthQueryBoundRequiredDomain>,
    pub(crate) commit_posture: WorthQueryBoundCommitPosture,
    pub(super) shape_proofs: super::authority_shape::WorthQueryBoundAuthorityShapeProofs,
}

static NEXT_BOUND_CAPABILITY_IDENTITY: AtomicU64 = AtomicU64::new(1);

pub struct WorthQueryBoundDomainOperation<D, O, F, L: BasisOperationLane> {
    operation: WorthQueryInstalledDomainOperation<D, O, F>,
    basis: super::WorthQueryOperatingWorldBasis<L>,
    execution_authority:
        worth_query_execution::facade::runtime::WorthQueryExecutionBoundOperationAuthority,
    graph_participations: Vec<WorthQueryBoundGraphParticipation>,
    required_domains: Vec<WorthQueryBoundRequiredDomain>,
    commit_posture: WorthQueryBoundCommitPosture,
    binding_counters: super::WorthQueryOperationBindingCounters,
    binding_identity: String,
    capability_identity: u64,
    authority_proof: WorthQueryOperationPhaseProof<WorthQueryBoundOperationPhase>,
    _authority_shape_proofs: super::authority_shape::WorthQueryBoundAuthorityShapeProofs,
    consumer_support_profile: WorthQueryConsumerSupportProfile,
    consumer_contract_minted: Cell<bool>,
    execution_providers: WorthQueryBoundExecutionProviders,
    conditional_nodes: Vec<Arc<super::super::WorthQueryInstalledConditionalNode>>,
    _marker: PhantomData<BoundOperationMarker<D, O, F>>,
}

impl<D, O, F, L: BasisOperationLane> WorthQueryBoundDomainOperation<D, O, F, L> {
    pub(super) fn mint(
        operation: WorthQueryInstalledDomainOperation<D, O, F>,
        basis: super::WorthQueryOperatingWorldBasis<L>,
        execution_authority:
            worth_query_execution::facade::runtime::WorthQueryExecutionBoundOperationAuthority,
        authorities: WorthQueryBoundAuthoritySet,
        consumer_support_profile: WorthQueryConsumerSupportProfile,
        execution_providers: WorthQueryBoundExecutionProviders,
        conditional_nodes: Vec<Arc<super::super::WorthQueryInstalledConditionalNode>>,
        binding_counters: super::WorthQueryOperationBindingCounters,
    ) -> Self {
        let required_domain_authority_identities = authorities
            .required_domains
            .iter()
            .map(|binding| {
                format!(
                    "{}:{}",
                    binding.role,
                    binding.authority.authority_identity().as_str()
                )
            })
            .collect::<Vec<_>>();
        let graph_authority_identities = authorities
            .graph_participations
            .iter()
            .map(|participation| {
                format!(
                    "{}:{}",
                    participation.role, participation.record.authority_identity
                )
            })
            .collect::<Vec<_>>();
        let capability_identity = NEXT_BOUND_CAPABILITY_IDENTITY.fetch_add(1, Ordering::Relaxed);
        let binding_identity = execution_authority.binding_identity().to_owned();
        let authority_proof = mint_operation_phase_proof(
            binding_identity.clone(),
            None,
            WorthQueryOperationAuthorityBasis {
                runtime_authority: operation.domain_authority().runtime_authority().as_u64(),
                installation_runtime_authority: operation.operation_authority().runtime_ordinal(),
                installation_generation: operation.operation_authority().generation().ordinal(),
                domain_authority_identity: operation
                    .domain_authority()
                    .authority_identity()
                    .as_str()
                    .into(),
                operation_identity: operation.definition().canonical_identity().into(),
                binding_identity: binding_identity.clone(),
                capability_identity,
                basis_identity: basis.lane().capability_digest().into(),
                graph_authority_identities,
                required_domain_authority_identities,
                resource_admission_identity: None,
            },
        );
        Self {
            operation,
            basis,
            execution_authority,
            graph_participations: authorities.graph_participations,
            required_domains: authorities.required_domains,
            commit_posture: authorities.commit_posture,
            binding_counters,
            binding_identity,
            capability_identity,
            authority_proof,
            _authority_shape_proofs: authorities.shape_proofs,
            consumer_support_profile,
            consumer_contract_minted: Cell::new(false),
            execution_providers,
            conditional_nodes,
            _marker: PhantomData,
        }
    }

    pub fn definition(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition {
        self.operation.definition()
    }

    pub fn basis_identity(&self) -> &str {
        self.basis.lane().capability_digest()
    }

    pub(crate) fn basis(&self) -> &AdmittedBasisCapability<L> {
        self.basis.lane()
    }

    pub(crate) fn product(
        &self,
    ) -> Result<
        &Arc<worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease>,
        super::super::WorthQueryConditionalAdmissionDenial,
    > {
        self.basis.product()
    }

    pub fn commit_posture(&self) -> WorthQueryBoundCommitPosture {
        self.commit_posture
    }

    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }

    pub(crate) fn execution_authority(
        &self,
    ) -> &worth_query_execution::facade::runtime::WorthQueryExecutionBoundOperationAuthority {
        &self.execution_authority
    }

    pub(crate) fn direct_domain_evidence_contract(
        &self,
    ) -> Option<
        std::sync::Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
    > {
        self.execution_authority
            .ordinary_direct_domain_evidence_contract()
            .cloned()
    }

    pub(crate) fn workflow_stage_domain_evidence_contract(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Option<
        std::sync::Arc<
            worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority,
        >,
    > {
        self.execution_authority
            .ordinary_workflow_domain_evidence_contract(stage.identity())
            .cloned()
    }

    pub const fn binding_counters(&self) -> super::WorthQueryOperationBindingCounters {
        self.binding_counters
    }

    pub fn graph_roles(&self) -> impl ExactSizeIterator<Item = &str> {
        self.graph_participations
            .iter()
            .map(|participation| participation.role.as_str())
    }

    pub fn required_domain_roles(&self) -> impl ExactSizeIterator<Item = &str> {
        self.required_domains
            .iter()
            .map(|binding| binding.role.as_str())
    }

    pub(crate) fn operation(&self) -> &WorthQueryInstalledDomainOperation<D, O, F> {
        &self.operation
    }

    pub(crate) fn installation_is_current(&self) -> bool {
        self.operation
            .domain_authority()
            .is_current_installation_generation()
    }

    pub(crate) fn capability_identity(&self) -> u64 {
        self.capability_identity
    }

    pub(crate) fn authority_proof(
        &self,
    ) -> &WorthQueryOperationPhaseProof<WorthQueryBoundOperationPhase> {
        &self.authority_proof
    }

    pub(crate) fn graph_participations(&self) -> &[WorthQueryBoundGraphParticipation] {
        &self.graph_participations
    }

    pub(crate) fn required_domains(&self) -> &[WorthQueryBoundRequiredDomain] {
        &self.required_domains
    }

    pub(crate) fn consumer_support_profile(&self) -> &WorthQueryConsumerSupportProfile {
        &self.consumer_support_profile
    }

    pub(crate) fn executor(&self) -> Option<&Arc<WorthQueryInstalledDomainOperationExecutor>> {
        match &self.execution_providers {
            WorthQueryBoundExecutionProviders::Direct { executor } => Some(executor),
            WorthQueryBoundExecutionProviders::Workflow { .. } => None,
        }
    }

    pub(crate) fn workflow_executor(
        &self,
    ) -> Option<&Arc<super::super::WorthQueryInstalledWorkflowStageExecutor>> {
        match &self.execution_providers {
            WorthQueryBoundExecutionProviders::Direct { .. } => None,
            WorthQueryBoundExecutionProviders::Workflow { executor, .. } => Some(executor),
        }
    }

    pub(crate) fn workflow_parallel_admission_provider(
        &self,
    ) -> Option<&Arc<super::super::WorthQueryInstalledWorkflowParallelAdmissionProvider>> {
        match &self.execution_providers {
            WorthQueryBoundExecutionProviders::Workflow {
                parallel: WorthQueryBoundWorkflowParallelPosture::Parallel(provider),
                ..
            } => Some(provider),
            _ => None,
        }
    }

    pub(crate) fn direct_executor(&self) -> &Arc<WorthQueryInstalledDomainOperationExecutor> {
        match &self.execution_providers {
            WorthQueryBoundExecutionProviders::Direct { executor } => executor,
            WorthQueryBoundExecutionProviders::Workflow { .. } => {
                unreachable!("bound direct operation retained workflow providers")
            }
        }
    }

    pub(crate) fn workflow_providers(
        &self,
    ) -> (
        &Arc<super::super::WorthQueryInstalledWorkflowGraph>,
        &Arc<super::super::WorthQueryInstalledWorkflowStageExecutor>,
        &WorthQueryBoundWorkflowParallelPosture,
    ) {
        match &self.execution_providers {
            WorthQueryBoundExecutionProviders::Direct { .. } => {
                unreachable!("bound workflow operation retained direct providers")
            }
            WorthQueryBoundExecutionProviders::Workflow {
                graph,
                executor,
                parallel,
            } => (graph, executor, parallel),
        }
    }

    pub(crate) fn conditional_nodes(
        &self,
    ) -> &[Arc<super::super::WorthQueryInstalledConditionalNode>] {
        &self.conditional_nodes
    }

    pub(crate) fn conditional_graph_authorities(
        &self,
    ) -> Vec<(
        &str,
        &Arc<worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority>,
    )> {
        self.graph_participations
            .iter()
            .map(|participation| {
                (
                    participation.role.as_str(),
                    &participation.record.installation_authority,
                )
            })
            .collect()
    }
}

impl<D, O, F, L: BasisOperationLane> WorthQueryBoundDomainOperation<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<D, F, Publication = WorthQueryPublishingOperation>,
{
    pub fn consumer_projection_contract(
        &self,
    ) -> Result<
        WorthQueryConsumerProjectionContract<D, O, F, L>,
        WorthQueryConsumerProjectionContractDenial,
    > {
        let mut counters = super::super::WorthQueryConsumerSupportAdmissionCounters::default();
        counters.installation_generation_checks += 1;
        if !self.installation_is_current() {
            return Err(
                WorthQueryConsumerProjectionContractDenial::StaleInstallationGeneration {
                    counters,
                },
            );
        }
        counters.mint_guard_checks += 1;
        if self.consumer_contract_minted.replace(true) {
            return Err(WorthQueryConsumerProjectionContractDenial::AlreadyMinted { counters });
        }
        WorthQueryConsumerProjectionContract::mint(self, &self.consumer_support_profile, counters)
            .map_err(Into::into)
    }
}
