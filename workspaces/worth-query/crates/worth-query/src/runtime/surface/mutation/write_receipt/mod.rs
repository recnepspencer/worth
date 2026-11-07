use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::memory_workspace::WorthQueryMutationReceipt;
use crate::runtime::mutation::WorthQueryMutationMetadata;

use super::{WorthQueryMutationFamily, WorthQueryWriteCommand};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::runtime::WorthQueryJournalPosition;
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryAspectTouch, WorthQueryAuthorityLane,
    WorthQueryContinuityMutationEvidence, WorthQueryContinuityMutationIntent,
    WorthQueryDerivedMaterializationTarget, WorthQueryExistingTruthAssertionEvidence,
    WorthQueryExistingTruthBindingEvidence, WorthQueryExistingTruthTargetBinding,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryIntentExecutionProvenance,
    WorthQueryLiveArtifactTarget, WorthQueryMutationCausalityEvidence,
    WorthQueryMutationProvenanceEvidence, WorthQueryMutationTargetCollectionIdentity,
    WorthQueryMutationTargetEvidence, WorthQueryNamingMutationEvidence,
    WorthQueryNamingMutationIntent, WorthQuerySymbolicAspectResolutionEvidence,
    WorthQuerySymbolicTargetReference, WorthQuerySymbolicTargetReferenceEvidence,
    WorthQueryVerifiedExistingTruthAssertion,
};

mod accessors;
mod identity;
use identity::{
    write_receipt_commit_evidence_identity, write_receipt_committed_truth_identity,
    write_receipt_snapshot_evidence_identity,
};
mod helpers;
mod preview;

use crate::memory_workspace::{
    WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQueryMutationKind,
    WorthQuerySnapshotIdentity,
};
use helpers::{
    assertion_evidence, continuity_mutation_evidence, naming_mutation_evidence,
    symbolic_target_reference_evidence, target_evidence_from_receipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWriteReceipt {
    pub(super) inner: WorthQueryMutationReceipt,
    pub(super) commit_evidence_identity: WorthQueryEvidenceIdentity,
    pub(super) committed_truth_identity: WorthQueryEvidenceIdentity,
    pub(super) journal_position: WorthQueryJournalPosition,
    pub(super) snapshot_evidence_identity: WorthQueryEvidenceIdentity,
    pub(super) mutation_family: WorthQueryMutationFamily,
    pub(super) authority_lane: WorthQueryAuthorityLane,
    pub(super) basis_lane: WorthQueryAuthorityLane,
    pub(super) target_evidence: WorthQueryMutationTargetEvidence,
    pub(super) existing_truth_assertion_evidence: Option<WorthQueryExistingTruthAssertionEvidence>,
    pub(super) existing_truth_binding_evidence: Option<WorthQueryExistingTruthBindingEvidence>,
    pub(super) symbolic_target_reference_evidence:
        Option<WorthQuerySymbolicTargetReferenceEvidence>,
    pub(super) symbolic_aspect_resolution_evidence: Vec<WorthQuerySymbolicAspectResolutionEvidence>,
    pub(super) naming_mutation_evidence: Option<WorthQueryNamingMutationEvidence>,
    pub(super) continuity_mutation_evidence: Option<WorthQueryContinuityMutationEvidence>,
    pub(super) causality_evidence: Option<WorthQueryMutationCausalityEvidence>,
    pub(super) provenance_evidence: Option<WorthQueryMutationProvenanceEvidence>,
    pub(super) declared_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
    pub(super) declared_entity_identity: Option<WorthQueryEntityIdentity>,
    pub(super) target_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
    pub(super) target_entity_identity: Option<WorthQueryEntityIdentity>,
    pub(super) declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
    pub(super) declared_aspect_value_digest: Option<WorthQueryEvidenceIdentity>,
    pub(super) mutation_metadata: WorthQueryMutationMetadata,
    pub(super) affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
    pub(super) affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
    pub(super) considered_computed_view_count: usize,
    pub(super) considered_effect_count: usize,
    pub(super) delivered_effect_count: usize,
    pub(super) pending_write_intent_count: usize,
    pub(super) suppressed_effect_count: usize,
    pub(super) meaningful_effect_suppression_count: usize,
    pub(super) effect_expression_failure_count: usize,
    pub(super) refresh_fallback: bool,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

impl WorthQueryWriteReceipt {
    pub(in crate::runtime) fn from_mutation_receipt(
        inner: WorthQueryMutationReceipt,
        mutation_family: WorthQueryMutationFamily,
        declared_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
        declared_entity_identity: Option<WorthQueryEntityIdentity>,
        existing_truth_binding: Option<WorthQueryExistingTruthTargetBinding>,
        existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
        symbolic_target_reference: Option<WorthQuerySymbolicTargetReference>,
        symbolic_aspect_resolution_evidence: Vec<WorthQuerySymbolicAspectResolutionEvidence>,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
        target_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
        target_entity_identity: Option<WorthQueryEntityIdentity>,
        declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
        declared_aspect_value_digest: Option<WorthQueryEvidenceIdentity>,
        mutation_metadata: WorthQueryMutationMetadata,
        affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
        affected_derived_view_targets: Vec<WorthQueryDerivedMaterializationTarget>,
        considered_computed_view_count: usize,
        considered_effect_count: usize,
        delivered_effect_count: usize,
        pending_write_intent_count: usize,
        suppressed_effect_count: usize,
        meaningful_effect_suppression_count: usize,
        effect_expression_failure_count: usize,
        refresh_fallback: bool,
        decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
        execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let commit_evidence_identity =
            write_receipt_commit_evidence_identity(&inner.commit_identity);
        let committed_truth_identity = write_receipt_committed_truth_identity(&inner)?;
        let journal_position =
            WorthQueryJournalPosition::from_commit_identity(&inner.commit_identity);
        let snapshot_evidence_identity =
            write_receipt_snapshot_evidence_identity(&inner.snapshot_identity);
        let target_evidence = target_evidence_from_receipt(
            mutation_family,
            declared_collection_identity.clone(),
            declared_entity_identity.clone(),
            target_collection_identity.clone(),
            target_entity_identity.clone(),
        );
        let causality_evidence = inner
            .bridge_authority
            .as_ref()
            .map(WorthQueryMutationCausalityEvidence::from_bridge)
            .transpose()?;
        let provenance_evidence = inner
            .bridge_authority
            .as_ref()
            .map(WorthQueryMutationProvenanceEvidence::from_bridge)
            .transpose()?;
        let existing_truth_binding_evidence = existing_truth_binding
            .as_ref()
            .map(WorthQueryExistingTruthBindingEvidence::from_binding)
            .or_else(|| {
                inner
                    .bridge_authority
                    .as_ref()
                    .and_then(|bundle| bundle.existing_truth_binding())
                    .map(WorthQueryExistingTruthBindingEvidence::from_bridge)
            })
            .transpose()?;
        let existing_truth_assertion_evidence = assertion_evidence(
            mutation_family,
            existing_truth_binding.as_ref(),
            existing_truth_assertion.as_ref(),
            &declared_aspect_operations,
            declared_aspect_value_digest.as_ref(),
            &inner.snapshot_identity,
        )?;
        let symbolic_target_reference_evidence = symbolic_target_reference_evidence(
            mutation_family,
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.symbolic_target_reference()),
            symbolic_target_reference.as_ref(),
            target_entity_identity.as_ref(),
        )?;
        let naming_mutation_evidence = naming_mutation_evidence(
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.naming_mutation()),
            naming_intent.as_ref(),
            target_entity_identity.as_ref(),
            target_collection_identity.as_ref(),
        )?;
        let continuity_mutation_evidence = continuity_mutation_evidence(
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.continuity_mutation()),
            continuity_intent.as_ref(),
            existing_truth_binding.as_ref(),
            target_entity_identity.as_ref(),
            target_collection_identity.as_ref(),
        )?;
        Ok(Self {
            inner,
            commit_evidence_identity,
            committed_truth_identity,
            journal_position,
            snapshot_evidence_identity,
            mutation_family,
            authority_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            basis_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
            target_evidence,
            existing_truth_assertion_evidence,
            existing_truth_binding_evidence,
            symbolic_target_reference_evidence,
            symbolic_aspect_resolution_evidence,
            naming_mutation_evidence,
            continuity_mutation_evidence,
            causality_evidence,
            provenance_evidence,
            declared_collection_identity,
            declared_entity_identity,
            target_collection_identity,
            target_entity_identity,
            declared_aspect_operations,
            declared_aspect_value_digest,
            mutation_metadata,
            affected_live_view_targets,
            affected_derived_view_targets,
            considered_computed_view_count,
            considered_effect_count,
            delivered_effect_count,
            pending_write_intent_count,
            suppressed_effect_count,
            meaningful_effect_suppression_count,
            effect_expression_failure_count,
            refresh_fallback,
            decision_trace_envelope,
            execution_provenance,
        })
    }

    pub(in crate::runtime) fn batch_component(
        inner: WorthQueryMutationReceipt,
        mutation_family: WorthQueryMutationFamily,
        basis_lane: WorthQueryAuthorityLane,
        declared_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
        declared_entity_identity: Option<WorthQueryEntityIdentity>,
        existing_truth_binding: Option<WorthQueryExistingTruthTargetBinding>,
        existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
        symbolic_target_reference: Option<WorthQuerySymbolicTargetReference>,
        symbolic_aspect_resolution_evidence: Vec<WorthQuerySymbolicAspectResolutionEvidence>,
        naming_intent: Option<WorthQueryNamingMutationIntent>,
        continuity_intent: Option<WorthQueryContinuityMutationIntent>,
        target_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
        target_entity_identity: Option<WorthQueryEntityIdentity>,
        declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
        declared_aspect_value_digest: Option<WorthQueryEvidenceIdentity>,
        mutation_metadata: WorthQueryMutationMetadata,
        affected_live_view_targets: Vec<WorthQueryLiveArtifactTarget>,
        authority_lane: WorthQueryAuthorityLane,
    ) -> Result<Self, IdentityDerivationDenial> {
        let commit_evidence_identity =
            write_receipt_commit_evidence_identity(&inner.commit_identity);
        let committed_truth_identity = write_receipt_committed_truth_identity(&inner)?;
        let journal_position =
            WorthQueryJournalPosition::from_commit_identity(&inner.commit_identity);
        let snapshot_evidence_identity =
            write_receipt_snapshot_evidence_identity(&inner.snapshot_identity);
        let target_evidence = target_evidence_from_receipt(
            mutation_family,
            declared_collection_identity.clone(),
            declared_entity_identity.clone(),
            target_collection_identity.clone(),
            target_entity_identity.clone(),
        );
        let causality_evidence = inner
            .bridge_authority
            .as_ref()
            .map(WorthQueryMutationCausalityEvidence::from_bridge)
            .transpose()?;
        let provenance_evidence = inner
            .bridge_authority
            .as_ref()
            .map(WorthQueryMutationProvenanceEvidence::from_bridge)
            .transpose()?;
        let existing_truth_binding_evidence = existing_truth_binding
            .as_ref()
            .map(WorthQueryExistingTruthBindingEvidence::from_binding)
            .or_else(|| {
                inner
                    .bridge_authority
                    .as_ref()
                    .and_then(|bundle| bundle.existing_truth_binding())
                    .map(WorthQueryExistingTruthBindingEvidence::from_bridge)
            })
            .transpose()?;
        let existing_truth_assertion_evidence = assertion_evidence(
            mutation_family,
            existing_truth_binding.as_ref(),
            existing_truth_assertion.as_ref(),
            &declared_aspect_operations,
            declared_aspect_value_digest.as_ref(),
            &inner.snapshot_identity,
        )?;
        let symbolic_target_reference_evidence = symbolic_target_reference_evidence(
            mutation_family,
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.symbolic_target_reference()),
            symbolic_target_reference.as_ref(),
            target_entity_identity.as_ref(),
        )?;
        let naming_mutation_evidence = naming_mutation_evidence(
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.naming_mutation()),
            naming_intent.as_ref(),
            target_entity_identity.as_ref(),
            target_collection_identity.as_ref(),
        )?;
        let continuity_mutation_evidence = continuity_mutation_evidence(
            inner
                .bridge_authority
                .as_ref()
                .and_then(|bundle| bundle.continuity_mutation()),
            continuity_intent.as_ref(),
            existing_truth_binding.as_ref(),
            target_entity_identity.as_ref(),
            target_collection_identity.as_ref(),
        )?;
        Ok(Self {
            inner,
            commit_evidence_identity,
            committed_truth_identity,
            journal_position,
            snapshot_evidence_identity,
            mutation_family,
            authority_lane,
            basis_lane,
            target_evidence,
            existing_truth_assertion_evidence,
            existing_truth_binding_evidence,
            symbolic_target_reference_evidence,
            symbolic_aspect_resolution_evidence,
            naming_mutation_evidence,
            continuity_mutation_evidence,
            causality_evidence,
            provenance_evidence,
            declared_collection_identity,
            declared_entity_identity,
            target_collection_identity,
            target_entity_identity,
            declared_aspect_operations,
            declared_aspect_value_digest,
            mutation_metadata,
            affected_live_view_targets,
            affected_derived_view_targets: Vec::new(),
            considered_computed_view_count: 0,
            considered_effect_count: 0,
            delivered_effect_count: 0,
            pending_write_intent_count: 0,
            suppressed_effect_count: 0,
            meaningful_effect_suppression_count: 0,
            effect_expression_failure_count: 0,
            refresh_fallback: false,
            decision_trace_envelope: None,
            execution_provenance: None,
        })
    }
}
