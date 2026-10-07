use crate::domain_installation::WorthQueryOperationResultState;
use std::collections::BTreeMap;

use super::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryWorkflowEffectEvidence,
    WorthQueryWorkflowInvariantOutcome, WorthQueryWorkflowPrimaryReadEvidence,
    WorthQueryWorkflowSemanticValue, WorthQueryWorkflowStageWarning, WorthQueryWorkflowValue,
};

#[derive(Debug)]
pub struct WorthQueryInstalledWorkflowGraph {
    definition: worth_query_installation::facade::WorthQueryPortableWorkflowDefinition,
    stage_index: BTreeMap<String, usize>,
}

impl WorthQueryInstalledWorkflowGraph {
    pub(crate) fn compile(
        operation: &worth_query_installation::facade::WorthQueryPortableDomainOperationDefinition,
    ) -> Option<Self> {
        match &operation.semantics().workflow {
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::Declared(
                definition,
            ) => {
                let definition = definition.clone();
                let stage_index = definition
                    .stages()
                    .iter()
                    .enumerate()
                    .map(|(index, stage)| (stage.identity().to_string(), index))
                    .collect();
                Some(Self {
                    definition,
                    stage_index,
                })
            }
            worth_query_installation::facade::WorthQueryOperationWorkflowContract::NotRequired => {
                None
            }
        }
    }

    pub fn entry_stage(&self) -> &str {
        self.definition.entry_stage()
    }

    pub fn stages(&self) -> &[worth_query_installation::facade::WorthQueryPortableWorkflowStage] {
        self.definition.stages()
    }

    pub(super) fn stage(
        &self,
        identity: &str,
    ) -> Option<&worth_query_installation::facade::WorthQueryPortableWorkflowStage> {
        self.stage_index
            .get(identity)
            .map(|index| &self.definition.stages()[*index])
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryWorkflowRunCounters {
    pub runtime_authority_checks: usize,
    pub stage_index_lookups: usize,
    pub stage_admission_checks: usize,
    pub predecessor_checks: usize,
    pub predecessor_receipt_lookups: usize,
    pub required_capability_checks: usize,
    pub required_domain_checks: usize,
    pub graph_read_contacts: usize,
    pub touch_effect_contacts: usize,
    pub effect_receipt_checks: usize,
    pub commit_admission_contacts: usize,
    pub invariant_checks: usize,
    pub parallel_admission_checks: usize,
    pub stage_executor_contacts: usize,
    /// Compute work settled by the execution authority, never refunded by apply.
    pub computation_charged_work: u64,
    pub output_contract_checks: usize,
    pub terminal_contract_checks: usize,
    pub consumption_contacts: usize,
    pub unrelated_run_scans: usize,
    pub conditional_request_admission_checks: usize,
    pub conditional_contract_lookups: usize,
    pub conditional_dependency_observation_reads: usize,
    pub conditional_dependency_checks: usize,
    pub conditional_semantic_reads: usize,
    pub conditional_condition_checks: usize,
    pub conditional_condition_deferrals: usize,
    pub conditional_temporal_deferrals: usize,
    pub conditional_on_demand_deferrals: usize,
    pub conditional_comparator_checks: usize,
    pub conditional_compute_contacts: usize,
    pub conditional_output_version_reads: usize,
    pub conditional_runtime_dependency_edges_captured: usize,
    pub conditional_application_contacts: usize,
    pub conditional_semantic_classifications: usize,
    pub conditional_reverted_clean_outcomes: usize,
    pub conditional_semantic_changes: usize,
    pub conditional_reuse_checks: usize,
    pub conditional_decisions_delivered: usize,
}

impl WorthQueryWorkflowRunCounters {
    pub(super) fn delta_since(self, before: Self) -> Self {
        Self {
            runtime_authority_checks: self.runtime_authority_checks
                - before.runtime_authority_checks,
            stage_index_lookups: self.stage_index_lookups - before.stage_index_lookups,
            stage_admission_checks: self.stage_admission_checks - before.stage_admission_checks,
            predecessor_checks: self.predecessor_checks - before.predecessor_checks,
            predecessor_receipt_lookups: self.predecessor_receipt_lookups
                - before.predecessor_receipt_lookups,
            required_capability_checks: self.required_capability_checks
                - before.required_capability_checks,
            required_domain_checks: self.required_domain_checks - before.required_domain_checks,
            graph_read_contacts: self.graph_read_contacts - before.graph_read_contacts,
            touch_effect_contacts: self.touch_effect_contacts - before.touch_effect_contacts,
            effect_receipt_checks: self.effect_receipt_checks - before.effect_receipt_checks,
            commit_admission_contacts: self.commit_admission_contacts
                - before.commit_admission_contacts,
            invariant_checks: self.invariant_checks - before.invariant_checks,
            parallel_admission_checks: self.parallel_admission_checks
                - before.parallel_admission_checks,
            stage_executor_contacts: self.stage_executor_contacts - before.stage_executor_contacts,
            computation_charged_work: self.computation_charged_work
                - before.computation_charged_work,
            output_contract_checks: self.output_contract_checks - before.output_contract_checks,
            terminal_contract_checks: self.terminal_contract_checks
                - before.terminal_contract_checks,
            consumption_contacts: self.consumption_contacts - before.consumption_contacts,
            unrelated_run_scans: self.unrelated_run_scans - before.unrelated_run_scans,
            conditional_request_admission_checks: self.conditional_request_admission_checks
                - before.conditional_request_admission_checks,
            conditional_contract_lookups: self.conditional_contract_lookups
                - before.conditional_contract_lookups,
            conditional_dependency_observation_reads: self.conditional_dependency_observation_reads
                - before.conditional_dependency_observation_reads,
            conditional_dependency_checks: self.conditional_dependency_checks
                - before.conditional_dependency_checks,
            conditional_semantic_reads: self.conditional_semantic_reads
                - before.conditional_semantic_reads,
            conditional_condition_checks: self.conditional_condition_checks
                - before.conditional_condition_checks,
            conditional_condition_deferrals: self.conditional_condition_deferrals
                - before.conditional_condition_deferrals,
            conditional_temporal_deferrals: self.conditional_temporal_deferrals
                - before.conditional_temporal_deferrals,
            conditional_on_demand_deferrals: self.conditional_on_demand_deferrals
                - before.conditional_on_demand_deferrals,
            conditional_comparator_checks: self.conditional_comparator_checks
                - before.conditional_comparator_checks,
            conditional_compute_contacts: self.conditional_compute_contacts
                - before.conditional_compute_contacts,
            conditional_output_version_reads: self.conditional_output_version_reads
                - before.conditional_output_version_reads,
            conditional_runtime_dependency_edges_captured: self
                .conditional_runtime_dependency_edges_captured
                - before.conditional_runtime_dependency_edges_captured,
            conditional_application_contacts: self.conditional_application_contacts
                - before.conditional_application_contacts,
            conditional_semantic_classifications: self.conditional_semantic_classifications
                - before.conditional_semantic_classifications,
            conditional_reverted_clean_outcomes: self.conditional_reverted_clean_outcomes
                - before.conditional_reverted_clean_outcomes,
            conditional_semantic_changes: self.conditional_semantic_changes
                - before.conditional_semantic_changes,
            conditional_reuse_checks: self.conditional_reuse_checks
                - before.conditional_reuse_checks,
            conditional_decisions_delivered: self.conditional_decisions_delivered
                - before.conditional_decisions_delivered,
        }
    }

    pub(super) fn observed_cost_roles(
        self,
    ) -> Vec<worth_query_installation::facade::WorthQueryWorkflowCostRole> {
        use worth_query_installation::facade::WorthQueryWorkflowCostRole as Role;
        let mut roles = vec![Role::Admission];
        if self.graph_read_contacts > 0 {
            roles.push(Role::GraphRead);
        }
        if self.touch_effect_contacts > 0 {
            roles.push(Role::TouchEffect);
        }
        if self.commit_admission_contacts > 0 {
            roles.push(Role::CommitAdmission);
        }
        if self.effect_receipt_checks > 0 {
            roles.push(Role::Effect);
        }
        if self.invariant_checks > 0 {
            roles.push(Role::Invariant);
        }
        if self.stage_executor_contacts > 0 {
            roles.push(Role::Execution);
        }
        if self.output_contract_checks > 0 || self.terminal_contract_checks > 0 {
            roles.push(Role::ResultValidation);
        }
        roles.sort();
        roles
    }
}

#[derive(Debug)]
pub struct WorthQueryWorkflowStageReceipt {
    pub(super) identity: String,
    pub(super) run_identity: String,
    pub(super) binding_identity: String,
    pub(super) stage_identity: String,
    pub(super) predecessor_stage_identities: Vec<String>,
    pub(super) predecessor_receipt_identities: Vec<String>,
    pub(super) input: WorthQueryWorkflowSemanticValue,
    pub(super) output: WorthQueryWorkflowValue,
    pub(super) output_semantics: WorthQueryWorkflowSemanticValue,
    pub(super) result_state: Option<WorthQueryOperationResultState>,
    pub(super) warnings: Vec<WorthQueryWorkflowStageWarning>,
    pub(super) graph_receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
    pub(super) primary_read_evidence: Vec<WorthQueryWorkflowPrimaryReadEvidence>,
    pub(super) effect_evidence: Vec<WorthQueryWorkflowEffectEvidence>,
    pub(super) invariant_outcomes: Vec<WorthQueryWorkflowInvariantOutcome>,
    pub(super) parallel_admission:
        Option<std::sync::Arc<super::WorthQueryWorkflowParallelAdmissionReceipt>>,
    pub(super) counters: WorthQueryWorkflowRunCounters,
    pub(super) authority_proof: std::sync::Arc<super::WorthQueryWorkflowRunAuthorityProof>,
    pub(super) stage_authority_proof: std::sync::Arc<WorthQueryWorkflowStageAuthorityProof>,
    pub(super) predecessor_authority_proofs:
        Vec<std::sync::Arc<WorthQueryWorkflowStageAuthorityProof>>,
    pub(super) execution_snapshot: crate::memory_workspace::WorthQuerySnapshotIdentity,
    pub(super) conditional: Vec<crate::domain_installation::WorthQueryConditionalProvenance>,
    pub(crate) lineage: Vec<crate::identity_evolution::InstalledIdentityEvolutionOutcome>,
    pub(super) domain_evidence: Option<super::WorthQueryAdmittedDomainEvidence>,
    pub(super) execution_resources: super::WorthQueryExecutionResourceAttemptEvidence,
    pub(super) artifact_provider_release: Option<
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderReleasePosture,
    >,
}

#[derive(Debug)]
pub(super) struct WorthQueryWorkflowStageAuthorityProof {
    pub(super) stage_identity: String,
    pub(super) run_authority: std::sync::Arc<super::WorthQueryWorkflowRunAuthorityProof>,
    pub(super) proof:
        crate::domain_installation::operation_authority_chain::WorthQueryOperationPhaseProof<
            crate::domain_installation::operation_authority_chain::WorthQueryWorkflowStagePhase,
        >,
}

impl WorthQueryWorkflowStageReceipt {
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }
    pub fn binding_identity(&self) -> &str {
        &self.binding_identity
    }
    pub fn execution_resources(&self) -> &super::WorthQueryExecutionResourceAttemptEvidence {
        &self.execution_resources
    }
    pub fn stage_identity(&self) -> &str {
        &self.stage_identity
    }
    pub fn predecessor_receipt_identities(&self) -> &[String] {
        &self.predecessor_receipt_identities
    }
    pub fn predecessor_stage_identities(&self) -> &[String] {
        &self.predecessor_stage_identities
    }
    pub fn predecessor_proof_count(&self) -> usize {
        self.predecessor_authority_proofs.len()
    }
    pub fn input(&self) -> &WorthQueryWorkflowSemanticValue {
        &self.input
    }
    pub(crate) fn output(&self) -> &WorthQueryWorkflowValue {
        &self.output
    }
    pub(super) fn take_output(&mut self) -> WorthQueryWorkflowValue {
        std::mem::replace(&mut self.output, WorthQueryWorkflowValue::NotRequired)
    }
    pub(super) fn restore_output(&mut self, output: WorthQueryWorkflowValue) {
        debug_assert!(matches!(self.output, WorthQueryWorkflowValue::NotRequired));
        self.output = output;
    }
    pub(super) fn set_artifact_disposition(
        &mut self,
        disposition: crate::domain_installation::WorthQueryArtifactDisposition,
    ) {
        self.output_semantics.set_artifact_disposition(disposition);
    }
    pub(super) fn cancel_artifact_output(&mut self) {
        let output = self.take_output();
        match output {
            WorthQueryWorkflowValue::InstalledArtifact(handle) => {
                let disposed = handle.cancel();
                self.set_artifact_disposition(
                    crate::domain_installation::WorthQueryArtifactDisposition::Cancelled,
                );
                self.artifact_provider_release = Some(disposed.provider_release());
            }
            output => self.restore_output(output),
        }
    }
    pub(super) fn retire_artifact_output(&mut self) {
        let output = self.take_output();
        match output {
            WorthQueryWorkflowValue::InstalledArtifact(handle) => {
                let disposed = handle.retire_for_trace();
                self.set_artifact_disposition(
                    crate::domain_installation::WorthQueryArtifactDisposition::Released,
                );
                self.artifact_provider_release = Some(disposed.provider_release());
            }
            output => self.restore_output(output),
        }
    }
    pub(crate) fn output_semantics(&self) -> &WorthQueryWorkflowSemanticValue {
        &self.output_semantics
    }
    pub fn artifact_provider_release(
        &self,
    ) -> Option<
        worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderReleasePosture,
    > {
        self.artifact_provider_release
    }
    pub fn result_state(&self) -> Option<WorthQueryOperationResultState> {
        self.result_state
    }
    pub fn warnings(&self) -> &[WorthQueryWorkflowStageWarning] {
        &self.warnings
    }
    pub fn graph_receipts(&self) -> &[WorthQueryBoundGraphExecutionReceipt] {
        &self.graph_receipts
    }
    pub fn primary_read_evidence(&self) -> &[WorthQueryWorkflowPrimaryReadEvidence] {
        &self.primary_read_evidence
    }
    pub fn effect_evidence(&self) -> &[WorthQueryWorkflowEffectEvidence] {
        &self.effect_evidence
    }
    pub fn invariant_outcomes(&self) -> &[WorthQueryWorkflowInvariantOutcome] {
        &self.invariant_outcomes
    }
    pub fn parallel_admission(&self) -> Option<&super::WorthQueryWorkflowParallelAdmissionReceipt> {
        self.parallel_admission.as_deref()
    }
    pub fn counters(&self) -> WorthQueryWorkflowRunCounters {
        self.counters
    }
    pub fn installation_generation(
        &self,
    ) -> crate::domain_installation::WorthQueryDomainInstallationGeneration {
        self.authority_proof
            .domain_authority
            .installation_generation()
    }
    pub fn operation_identity(&self) -> &str {
        self.authority_proof.operation_identity()
    }
    pub fn basis_identity(&self) -> &str {
        self.authority_proof.basis_identity()
    }
    pub fn execution_snapshot(&self) -> &crate::memory_workspace::WorthQuerySnapshotIdentity {
        &self.execution_snapshot
    }
    pub fn conditional_provenance(
        &self,
    ) -> &[crate::domain_installation::WorthQueryConditionalProvenance] {
        &self.conditional
    }
    pub fn domain_evidence(&self) -> Option<&super::WorthQueryAdmittedDomainEvidence> {
        self.domain_evidence.as_ref()
    }
}
