use super::batch_write_digest::{
    build_batch_write_receipt_inspection_digest, WorthQueryBatchWriteDigestInputs,
};
use super::WorthQueryBatchWriteComponentInspection;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryCommitIdentity;
use crate::runtime::{
    WorthQueryAspectTouch, WorthQueryAuthorityLane, WorthQueryBatchMutationEvidence,
    WorthQueryBatchWriteReceipt, WorthQueryGraphCompositionAssumptionSummary,
    WorthQueryGraphCompositionBreadth, WorthQueryGraphCompositionEvidence,
    WorthQueryGraphCompositionLifecycleOutcomes, WorthQueryGraphCompositionLineageSummary,
    WorthQueryGraphCompositionProgram, WorthQueryGraphCompositionResolutionMap,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBatchWriteReceiptInspection {
    authority_lane: WorthQueryAuthorityLane,
    basis_lane: WorthQueryAuthorityLane,
    batch_digest: WorthQueryEvidenceIdentity,
    batch_mutation_evidence: WorthQueryBatchMutationEvidence,
    graph_composition_breadth: WorthQueryGraphCompositionBreadth,
    graph_composition_lifecycle_outcomes: Option<WorthQueryGraphCompositionLifecycleOutcomes>,
    graph_composition_program: Option<WorthQueryGraphCompositionProgram>,
    graph_composition_evidence: Option<WorthQueryGraphCompositionEvidence>,
    graph_composition_resolution_map: WorthQueryGraphCompositionResolutionMap,
    write_receipt_count: usize,
    commit_identities: Vec<WorthQueryCommitIdentity>,
    journal_position_identities: Vec<WorthQueryEvidenceIdentity>,
    component_operations: Vec<WorthQueryBatchWriteComponentInspection>,
    touched_aspects: Vec<WorthQueryAspectTouch>,
    affected_live_view_ids: Vec<String>,
    affected_derived_view_ids: Vec<String>,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryBatchWriteReceiptInspection {
    pub(in crate::runtime) fn new(
        receipt: &WorthQueryBatchWriteReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let commit_identities = receipt
            .write_receipts()
            .iter()
            .map(|entry| entry.commit_identity().clone())
            .collect::<Vec<_>>();
        let component_operations = receipt
            .write_receipts()
            .iter()
            .map(WorthQueryBatchWriteComponentInspection::from_write_receipt)
            .collect::<Vec<_>>();
        let journal_position_identities = receipt
            .journal_positions()
            .map(|position| position.evidence_identity())
            .collect::<Vec<_>>();
        let batch_mutation_evidence = receipt.batch_mutation_evidence().clone();
        let graph_composition_breadth = receipt.graph_composition_breadth().clone();
        let graph_composition_lifecycle_outcomes = receipt.graph_composition_lifecycle_outcomes();
        let graph_composition_program = receipt.graph_composition_program().cloned();
        let graph_composition_evidence = receipt.graph_composition_evidence();
        let graph_composition_resolution_map = receipt.graph_composition_resolution_map().clone();
        let touched_aspects = receipt.admitted_touched_aspects().to_vec();
        let affected_live_view_ids = receipt.terminal_affected_live_view_ids_projection();
        let affected_derived_view_ids = receipt.terminal_affected_derived_view_ids_projection();
        let inspection_identity =
            build_batch_write_receipt_inspection_digest(WorthQueryBatchWriteDigestInputs {
                authority_lane: receipt.authority_lane().as_str(),
                basis_lane: receipt.basis_lane().as_str(),
                batch_digest: receipt.batch_identity(),
                graph_composition_breadth: &graph_composition_breadth,
                graph_composition_lifecycle_outcomes: graph_composition_lifecycle_outcomes.as_ref(),
                graph_composition_program: graph_composition_program.as_ref(),
                graph_composition_evidence: graph_composition_evidence.as_ref(),
                batch_mutation_evidence: &batch_mutation_evidence,
                commit_identities: &commit_identities,
                journal_position_identities: &journal_position_identities,
                component_operations: &component_operations,
                graph_composition_resolution_map: &graph_composition_resolution_map,
                touched_aspects: &touched_aspects,
                affected_live_view_ids: &affected_live_view_ids,
                affected_derived_view_ids: &affected_derived_view_ids,
            })?;
        Ok(Self {
            authority_lane: receipt.authority_lane(),
            basis_lane: receipt.basis_lane(),
            batch_digest: receipt.batch_identity().clone(),
            batch_mutation_evidence,
            graph_composition_breadth,
            graph_composition_lifecycle_outcomes,
            graph_composition_program,
            graph_composition_evidence,
            graph_composition_resolution_map,
            write_receipt_count: receipt.write_count(),
            commit_identities,
            journal_position_identities,
            component_operations,
            touched_aspects,
            affected_live_view_ids,
            affected_derived_view_ids,
            inspection_identity,
        })
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn basis_lane(&self) -> WorthQueryAuthorityLane {
        self.basis_lane
    }

    pub fn batch_digest(&self) -> &str {
        self.batch_digest.as_str()
    }

    pub fn batch_mutation_evidence(&self) -> &WorthQueryBatchMutationEvidence {
        &self.batch_mutation_evidence
    }

    pub fn graph_composition_breadth(&self) -> &WorthQueryGraphCompositionBreadth {
        &self.graph_composition_breadth
    }

    pub fn graph_composition_lifecycle_outcomes(
        &self,
    ) -> Option<&WorthQueryGraphCompositionLifecycleOutcomes> {
        self.graph_composition_lifecycle_outcomes.as_ref()
    }

    pub fn graph_composition_program(&self) -> Option<&WorthQueryGraphCompositionProgram> {
        self.graph_composition_program.as_ref()
    }

    pub fn graph_composition_evidence(&self) -> Option<&WorthQueryGraphCompositionEvidence> {
        self.graph_composition_evidence.as_ref()
    }

    pub fn graph_composition_assumption_summary(
        &self,
    ) -> Option<&WorthQueryGraphCompositionAssumptionSummary> {
        self.graph_composition_evidence
            .as_ref()
            .and_then(WorthQueryGraphCompositionEvidence::assumption_summary)
    }

    pub fn graph_composition_lineage_summary(
        &self,
    ) -> Option<&WorthQueryGraphCompositionLineageSummary> {
        self.graph_composition_evidence
            .as_ref()
            .and_then(WorthQueryGraphCompositionEvidence::lineage_summary)
    }

    pub fn graph_composition_resolution_map(&self) -> &WorthQueryGraphCompositionResolutionMap {
        &self.graph_composition_resolution_map
    }

    pub fn write_receipt_count(&self) -> usize {
        self.write_receipt_count
    }

    pub fn commit_identities(&self) -> &[WorthQueryCommitIdentity] {
        &self.commit_identities
    }

    pub fn journal_position_identities(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.journal_position_identities
    }

    pub fn component_operations(&self) -> &[WorthQueryBatchWriteComponentInspection] {
        &self.component_operations
    }

    pub fn admitted_touched_aspects(&self) -> &[WorthQueryAspectTouch] {
        &self.touched_aspects
    }

    pub fn terminal_affected_live_view_ids_projection(&self) -> &[String] {
        &self.affected_live_view_ids
    }

    pub fn terminal_affected_derived_view_ids_projection(&self) -> &[String] {
        &self.affected_derived_view_ids
    }

    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
