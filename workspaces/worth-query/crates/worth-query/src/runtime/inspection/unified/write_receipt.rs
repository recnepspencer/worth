use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod digest;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQuerySnapshotIdentity,
};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAuthorityLane,
    WorthQueryContinuityMutationEvidence, WorthQueryExistingTruthAssertionEvidence,
    WorthQueryExistingTruthBindingEvidence, WorthQueryInspectedArtifact,
    WorthQueryMutationCausalityEvidence, WorthQueryMutationMetadata,
    WorthQueryMutationProvenanceEvidence, WorthQueryMutationTargetEvidence,
    WorthQueryNamingMutationEvidence, WorthQueryRuntimeInspectionEvidence,
    WorthQuerySymbolicTargetReferenceEvidence, WorthQueryWriteReceipt,
};
use digest::build_write_receipt_inspection_digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWriteReceiptInspection {
    mutation_family: String,
    authority_lane: WorthQueryAuthorityLane,
    basis_lane: WorthQueryAuthorityLane,
    target_evidence: WorthQueryMutationTargetEvidence,
    existing_truth_assertion_evidence: Option<WorthQueryExistingTruthAssertionEvidence>,
    existing_truth_binding_evidence: Option<WorthQueryExistingTruthBindingEvidence>,
    symbolic_target_reference_evidence: Option<WorthQuerySymbolicTargetReferenceEvidence>,
    naming_mutation_evidence: Option<WorthQueryNamingMutationEvidence>,
    continuity_mutation_evidence: Option<WorthQueryContinuityMutationEvidence>,
    causality_evidence: Option<WorthQueryMutationCausalityEvidence>,
    provenance_evidence: Option<WorthQueryMutationProvenanceEvidence>,
    declared_collection: Option<String>,
    declared_entity_identity: Option<WorthQueryEntityIdentity>,
    target_collection: Option<String>,
    target_entity_identity: Option<WorthQueryEntityIdentity>,
    commit_identity: WorthQueryCommitIdentity,
    snapshot_identity: WorthQuerySnapshotIdentity,
    canonical_artifact: WorthQueryInspectedArtifact,
    workflow_artifact: WorthQueryInspectedArtifact,
    bridge_authority_artifact: WorthQueryInspectedArtifact,
    runtime_evidence: WorthQueryRuntimeInspectionEvidence,
    live_patch_artifacts: Vec<WorthQueryEvidenceIdentity>,
    declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
    declared_aspect_value_digest: Option<WorthQueryEvidenceIdentity>,
    mutation_metadata: WorthQueryMutationMetadata,
    inspection_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryWriteReceiptInspection {
    pub(in crate::runtime) fn new(
        receipt: &WorthQueryWriteReceipt,
        runtime_evidence: WorthQueryRuntimeInspectionEvidence,
    ) -> Result<Self, IdentityDerivationDenial> {
        let canonical_artifact = WorthQueryInspectedArtifact::new(
            "canonical",
            receipt.commit_evidence_identity().clone(),
            receipt.snapshot_evidence_identity().clone(),
        );
        let workflow_artifact = WorthQueryInspectedArtifact::new(
            "workflow",
            receipt.commit_evidence_identity().clone(),
            receipt.snapshot_evidence_identity().clone(),
        );
        let bridge_authority_artifact = WorthQueryInspectedArtifact::new(
            "bridge-authority",
            receipt.commit_evidence_identity().clone(),
            receipt.snapshot_evidence_identity().clone(),
        );
        let live_patch_artifacts = receipt
            .deltas()
            .iter()
            .map(|delta| {
                worth_query_evidence_identity(
                    WorthQueryEvidenceScope::WriteReceiptInspectionArtifact,
                )
                .field_shape(WorthQueryEvidenceTag::new("role"), "live-patch-artifact")
                .field_value(WorthQueryEvidenceTag::new("collection"), delta.collection())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("entity_identity"),
                    &delta.entity_identity.evidence_identity(),
                )
                .seal()
            })
            .collect::<Vec<_>>();
        let declared_aspect_operations = receipt.declared_aspect_operations().to_vec();
        let declared_aspect_value_digest = receipt.declared_aspect_value_identity().cloned();
        let mutation_metadata = receipt.mutation_metadata().clone();
        let target_evidence = receipt.target_evidence().clone();
        let existing_truth_assertion_evidence =
            receipt.existing_truth_assertion_evidence().cloned();
        let existing_truth_binding_evidence = receipt.existing_truth_binding_evidence().cloned();
        let symbolic_target_reference_evidence =
            receipt.symbolic_target_reference_evidence().cloned();
        let naming_mutation_evidence = receipt.naming_mutation_evidence().cloned();
        let continuity_mutation_evidence = receipt.continuity_mutation_evidence().cloned();
        let causality_evidence = receipt.causality_evidence().cloned();
        let provenance_evidence = receipt.provenance_evidence().cloned();
        let inspection_identity = build_write_receipt_inspection_digest(
            receipt,
            &target_evidence,
            existing_truth_assertion_evidence.as_ref(),
            existing_truth_binding_evidence.as_ref(),
            symbolic_target_reference_evidence.as_ref(),
            naming_mutation_evidence.as_ref(),
            continuity_mutation_evidence.as_ref(),
            causality_evidence.as_ref(),
            provenance_evidence.as_ref(),
            &runtime_evidence,
            &declared_aspect_operations,
            declared_aspect_value_digest.as_ref(),
            &mutation_metadata,
            &live_patch_artifacts,
        )?;
        Ok(Self {
            mutation_family: receipt.mutation_family().as_str().to_string(),
            authority_lane: receipt.authority_lane(),
            basis_lane: receipt.basis_lane(),
            target_evidence,
            existing_truth_assertion_evidence,
            existing_truth_binding_evidence,
            symbolic_target_reference_evidence,
            naming_mutation_evidence,
            continuity_mutation_evidence,
            causality_evidence,
            provenance_evidence,
            declared_collection: receipt
                .terminal_declared_collection_projection()
                .map(str::to_string),
            declared_entity_identity: receipt.declared_entity_identity().cloned(),
            target_collection: receipt
                .terminal_target_collection_projection()
                .map(str::to_string),
            target_entity_identity: receipt.target_entity_identity().cloned(),
            commit_identity: receipt.commit_identity().clone(),
            snapshot_identity: receipt.snapshot_identity().clone(),
            canonical_artifact,
            workflow_artifact,
            bridge_authority_artifact,
            runtime_evidence,
            live_patch_artifacts,
            declared_aspect_operations,
            declared_aspect_value_digest,
            mutation_metadata,
            inspection_identity,
        })
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn mutation_family(&self) -> &str {
        &self.mutation_family
    }

    pub fn basis_lane(&self) -> WorthQueryAuthorityLane {
        self.basis_lane
    }

    pub fn target_evidence(&self) -> &WorthQueryMutationTargetEvidence {
        &self.target_evidence
    }

    pub fn causality_evidence(&self) -> Option<&WorthQueryMutationCausalityEvidence> {
        self.causality_evidence.as_ref()
    }

    pub fn existing_truth_assertion_evidence(
        &self,
    ) -> Option<&WorthQueryExistingTruthAssertionEvidence> {
        self.existing_truth_assertion_evidence.as_ref()
    }

    pub fn verified_assumption_set(
        &self,
    ) -> Option<&crate::runtime::WorthQueryVerifiedAssumptionSet> {
        self.existing_truth_assertion_evidence
            .as_ref()
            .and_then(WorthQueryExistingTruthAssertionEvidence::verified_assumption_set)
    }

    pub fn verification_read_set_breadth(
        &self,
    ) -> Option<&crate::runtime::WorthQueryVerificationReadSetBreadth> {
        self.existing_truth_assertion_evidence
            .as_ref()
            .and_then(WorthQueryExistingTruthAssertionEvidence::verification_read_set_breadth)
    }

    pub fn existing_truth_binding_evidence(
        &self,
    ) -> Option<&WorthQueryExistingTruthBindingEvidence> {
        self.existing_truth_binding_evidence.as_ref()
    }

    pub fn symbolic_target_reference_evidence(
        &self,
    ) -> Option<&WorthQuerySymbolicTargetReferenceEvidence> {
        self.symbolic_target_reference_evidence.as_ref()
    }

    pub fn naming_mutation_evidence(&self) -> Option<&WorthQueryNamingMutationEvidence> {
        self.naming_mutation_evidence.as_ref()
    }

    pub fn continuity_mutation_evidence(&self) -> Option<&WorthQueryContinuityMutationEvidence> {
        self.continuity_mutation_evidence.as_ref()
    }

    pub fn provenance_evidence(&self) -> Option<&WorthQueryMutationProvenanceEvidence> {
        self.provenance_evidence.as_ref()
    }

    pub fn declared_collection(&self) -> Option<&str> {
        self.declared_collection.as_deref()
    }

    pub fn declared_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.declared_entity_identity.as_ref()
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection.as_deref()
    }

    pub fn target_entity_identity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.target_entity_identity.as_ref()
    }

    pub fn commit_identity(&self) -> &WorthQueryCommitIdentity {
        &self.commit_identity
    }

    pub fn snapshot_identity(&self) -> &WorthQuerySnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn canonical_artifact(&self) -> &WorthQueryInspectedArtifact {
        &self.canonical_artifact
    }

    pub fn workflow_artifact(&self) -> &WorthQueryInspectedArtifact {
        &self.workflow_artifact
    }

    pub fn bridge_authority_artifact(&self) -> &WorthQueryInspectedArtifact {
        &self.bridge_authority_artifact
    }

    pub fn runtime_evidence(&self) -> &WorthQueryRuntimeInspectionEvidence {
        &self.runtime_evidence
    }

    pub fn live_patch_artifacts(&self) -> &[WorthQueryEvidenceIdentity] {
        &self.live_patch_artifacts
    }

    pub fn declared_aspect_operations(&self) -> &[WorthQueryAspectMutationOperation] {
        &self.declared_aspect_operations
    }

    pub fn declared_aspect_value_digest(&self) -> Option<&str> {
        self.declared_aspect_value_digest
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn mutation_metadata(&self) -> &WorthQueryMutationMetadata {
        &self.mutation_metadata
    }

    pub fn inspection_digest(&self) -> &str {
        self.inspection_identity.as_str()
    }

    pub fn inspection_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.inspection_identity
    }
}
