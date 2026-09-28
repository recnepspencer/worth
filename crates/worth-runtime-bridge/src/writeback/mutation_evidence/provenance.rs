use std::sync::Arc;

use super::super::{
    BridgeDerivedWritebackEffect, BridgeWritebackAuthorityOutcome, BridgeWritebackExecutionRecord,
    BridgeWritebackFailureClass, BridgeWritebackFeedbackProvenance,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackOutcomeClass,
    BridgeWritebackStrategyDescriptorBasis,
};
use crate::adapter::{TruthWritebackReceipt, TruthWritebackRequest};
use crate::input::envelope::TruthCommitIdentity;
use crate::snapshot::TruthSnapshotIdentity;
use crate::writeback::{
    BridgeMutationSubject, BridgeMutationSubjectKind, BridgeMutationSubjectTouch,
};

/// Bridge-owned causality breadcrumbs for one authoritative mutation crossing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMutationCausalityBundle {
    truth_trigger_identity: TruthCommitIdentity,
    truth_view_snapshot_identity: TruthSnapshotIdentity,
    mutation_subject: Option<BridgeMutationSubject>,
    causality_digest: Arc<str>,
    truth_trigger_digest: Arc<str>,
    route_digest: Arc<str>,
    evaluation_surface_digest: Arc<str>,
    truth_view_digest: Arc<str>,
}

impl BridgeMutationCausalityBundle {
    pub fn from_writeback_causality(causality: &BridgeWritebackNativeCausalityInputs) -> Self {
        Self {
            truth_trigger_identity: causality.truth_trigger_identity().clone(),
            truth_view_snapshot_identity: causality.truth_view_snapshot_identity().clone(),
            mutation_subject: causality.mutation_subject().cloned(),
            causality_digest: Arc::from(causality.digest().to_owned()),
            truth_trigger_digest: Arc::from(causality.truth_trigger_digest().to_owned()),
            route_digest: Arc::from(causality.route_digest().to_owned()),
            evaluation_surface_digest: Arc::from(causality.evaluation_surface_digest().to_owned()),
            truth_view_digest: Arc::from(causality.truth_view_digest().to_owned()),
        }
    }

    pub fn causality_digest(&self) -> &str {
        self.causality_digest.as_ref()
    }

    pub fn truth_trigger_digest(&self) -> &str {
        self.truth_trigger_digest.as_ref()
    }

    pub fn route_digest(&self) -> &str {
        self.route_digest.as_ref()
    }

    pub fn evaluation_surface_digest(&self) -> &str {
        self.evaluation_surface_digest.as_ref()
    }

    pub fn truth_view_digest(&self) -> &str {
        self.truth_view_digest.as_ref()
    }

    pub fn retains_truth_handoff(
        &self,
        commit: &TruthCommitIdentity,
        snapshot: &TruthSnapshotIdentity,
    ) -> bool {
        &self.truth_trigger_identity == commit && &self.truth_view_snapshot_identity == snapshot
    }

    pub fn retains_mutation_subject(
        &self,
        target_collection: &str,
        target_record: crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts,
        mutation_kind: BridgeMutationSubjectKind,
        touches: &[BridgeMutationSubjectTouch],
    ) -> bool {
        self.mutation_subject.as_ref().is_some_and(|subject| {
            subject.matches_projection(target_collection, target_record, mutation_kind, touches)
        })
    }
}

/// Bridge-owned provenance breadcrumbs for one authoritative mutation crossing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeMutationProvenanceBundle {
    contract_digest: Arc<str>,
    writeback_effect_artifact_digest: Arc<str>,
    effect_intent_digest: Arc<str>,
    effect_intent_patch_canonical_basis: Arc<str>,
    feedback_provenance_digest: Arc<str>,
    causality_digest: Arc<str>,
    strategy_descriptor_basis: BridgeWritebackStrategyDescriptorBasis,
    execution_record_digest: Arc<str>,
    outcome_class: Option<BridgeWritebackOutcomeClass>,
    authoritative_artifact_digest: Option<Arc<str>>,
    authority_request: Option<TruthWritebackRequest>,
    authority_receipt: Option<TruthWritebackReceipt>,
    failure_class: Option<BridgeWritebackFailureClass>,
}

impl BridgeMutationProvenanceBundle {
    pub fn from_writeback_artifacts(
        effect: &BridgeDerivedWritebackEffect,
        feedback: &BridgeWritebackFeedbackProvenance,
        execution_record: &BridgeWritebackExecutionRecord,
        outcome: Option<&BridgeWritebackAuthorityOutcome>,
    ) -> Self {
        Self {
            contract_digest: Arc::from(execution_record.contract_digest().to_owned()),
            writeback_effect_artifact_digest: Arc::from(
                execution_record
                    .writeback_effect_artifact_digest()
                    .to_owned(),
            ),
            effect_intent_digest: Arc::from(execution_record.effect_intent_digest().to_owned()),
            effect_intent_patch_canonical_basis: Arc::from(
                execution_record
                    .effect_intent_patch_canonical_basis()
                    .to_owned(),
            ),
            feedback_provenance_digest: Arc::from(feedback.digest().to_owned()),
            causality_digest: Arc::from(execution_record.causality_digest().to_owned()),
            strategy_descriptor_basis: effect.strategy_descriptor_basis().clone(),
            execution_record_digest: Arc::from(execution_record.digest().to_owned()),
            outcome_class: execution_record.outcome_class(),
            authoritative_artifact_digest: outcome
                .map(|value| Arc::from(value.authoritative_artifact_digest().to_owned())),
            authority_request: execution_record.authority_request().cloned(),
            authority_receipt: execution_record.authority_receipt().cloned(),
            failure_class: execution_record.failure_class(),
        }
    }

    pub fn contract_digest(&self) -> &str {
        self.contract_digest.as_ref()
    }

    pub fn writeback_effect_artifact_digest(&self) -> &str {
        self.writeback_effect_artifact_digest.as_ref()
    }

    pub fn effect_intent_digest(&self) -> &str {
        self.effect_intent_digest.as_ref()
    }

    pub fn effect_intent_patch_canonical_basis(&self) -> &str {
        self.effect_intent_patch_canonical_basis.as_ref()
    }

    pub fn feedback_provenance_digest(&self) -> &str {
        self.feedback_provenance_digest.as_ref()
    }

    pub fn causality_digest(&self) -> &str {
        self.causality_digest.as_ref()
    }

    pub fn strategy_descriptor_basis(&self) -> &BridgeWritebackStrategyDescriptorBasis {
        &self.strategy_descriptor_basis
    }

    pub fn strategy_descriptor_digest(&self) -> &str {
        self.strategy_descriptor_basis.digest()
    }

    pub fn execution_record_digest(&self) -> &str {
        self.execution_record_digest.as_ref()
    }

    pub fn outcome_class(&self) -> Option<BridgeWritebackOutcomeClass> {
        self.outcome_class
    }

    pub fn authoritative_artifact_digest(&self) -> Option<&str> {
        self.authoritative_artifact_digest.as_deref()
    }

    pub fn request_digest(&self) -> Option<&str> {
        self.authority_request
            .as_ref()
            .map(TruthWritebackRequest::digest)
    }

    pub fn receipt_digest(&self) -> Option<&str> {
        self.authority_receipt
            .as_ref()
            .map(TruthWritebackReceipt::digest)
    }

    pub fn authority_request(&self) -> Option<&TruthWritebackRequest> {
        self.authority_request.as_ref()
    }

    pub fn authority_receipt(&self) -> Option<&TruthWritebackReceipt> {
        self.authority_receipt.as_ref()
    }

    pub fn failure_class(&self) -> Option<BridgeWritebackFailureClass> {
        self.failure_class
    }
}
