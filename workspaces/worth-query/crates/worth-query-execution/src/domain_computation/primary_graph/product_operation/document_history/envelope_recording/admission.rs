//! One envelope reserves one history record before any candidate exists.
use super::super::{
    journal::AttemptLookup, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryHistoryNodeId,
    WorthQueryDocumentHistoryRecordPreparation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryStepMetadata,
};
use crate::{
    basis::WorthQueryProductBranchReadIdentity,
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyBinding,
        WorthQueryApplicationProjection,
    },
    publication_boundary::{
        document_history::WorthQueryDocumentHistoryRead, WorthQueryProgramPublicationAccess,
    },
};
use sha2::{Digest, Sha256};
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryAccessBinding, ApplicationHistoryBinding},
    application_operation::{ApplicationEditIntentIdentity, ApplicationMutationBindingDescriptor},
    application_query::ApplicationQueryBinding,
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution_lineage::facade::ports::IdempotencyBindingInternalPort;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_query::facade::ports::HistoryNodeIdInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

type Access<S, H> = ApplicationHistoryAccessBinding<S, H>;
type Query<S, H> = <Access<S, H> as ApplicationQueryBinding<S>>::Query;
type Rows<S, H> = <<Access<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// What one envelope means to the history journal: its caller key, its complete
/// ordered intent, and the first stage's operation, which names the step.
pub struct WorthQueryDocumentHistoryEnvelopeIntent<'a> {
    key: &'a [u8; 32],
    intent: ApplicationEditIntentIdentity,
    first_operation: &'a ApplicationMutationBindingDescriptor,
    correction: Option<WorthQueryDocumentHistoryCorrectionMeaning>,
}

/// What a correction envelope's meaning adds to its stages (plan §11.1): the
/// correction binding, the recorded step it reverses, and which of that
/// step's stages (its commit member, public #368). Two sources whose derived
/// inverse inputs are equal are two meanings, as in the model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDocumentHistoryCorrectionMeaning {
    pub binding: &'static str,
    pub source: WorthQueryHistoryNodeId,
    pub member: String,
}

impl<'a> WorthQueryDocumentHistoryEnvelopeIntent<'a> {
    pub fn new(
        key: &'a [u8; 32],
        intent: ApplicationEditIntentIdentity,
        first_operation: &'a ApplicationMutationBindingDescriptor,
    ) -> Self {
        Self {
            key,
            intent,
            first_operation,
            correction: None,
        }
    }

    /// The same envelope, meaning a correction of `correction.source`.
    pub fn with_correction(mut self, correction: WorthQueryDocumentHistoryCorrectionMeaning) -> Self {
        self.correction = Some(correction);
        self
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// The recorded step on the selected line whose stages made `source`, for
    /// a correction naming it; refused before any reservation otherwise.
    pub fn selected_line_correction_source(
        &self,
        source: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> Result<WorthQueryHistoryNodeId, Denial> {
        self.selection
            .lock()
            .journal
            .selected_line_source(source)
            .ok_or_else(|| Denial::SourceNotOnSelectedLine { source: source.clone() })
    }

    /// This document's answer about its own navigation commits, for a
    /// correction's restored support on its candidate (S3 ruling B).
    pub fn correction_navigation_lineage(
        &self,
    ) -> std::sync::Arc<dyn crate::domain_computation::primary_graph::product_operation::WorthQueryCorrectionNavigationLineage + 'runtime> {
        std::sync::Arc::new(DocumentNavigationLineage {
            selection: self.selection.downgrade(),
        })
    }

    /// Reserve the one record an envelope will complete. Nothing is forked here:
    /// a refusal or an already recorded step leaves the document untouched.
    pub fn reserve_history_envelope(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        envelope: WorthQueryDocumentHistoryEnvelopeIntent<'_>,
        step_metadata: &WorthQueryHistoryStepMetadata,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryDocumentHistoryRecordPreparation, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let runtime = self.application();
        // Every fresh attempt lookup is metadata disclosure, even when the result
        // names a previously completed step rather than executing again.
        runtime.read_document_history_step(phase, access, self, read)?;
        self.selection
            .lock()
            .journal
            .admit_step_metadata(step_metadata)?;
        let label = installed
            .operation_descriptors()
            .iter()
            .zip(&installed.definition().parts().operations)
            .find(|(descriptor, _)| *descriptor == envelope.first_operation)
            .map(|(_, operation)| operation.label.as_str())
            .ok_or(Denial::OperationNotDeclared)?;
        let mut key_hash = Sha256::new();
        key_hash.update(b"worth.query.document-history.envelope.key.v1");
        key_hash.update(H::IDENTITY.as_bytes());
        key_hash.update(envelope.key);
        let mut intent_hash = Sha256::new();
        match &envelope.correction {
            // An edit keeps Phase 1's meaning byte for byte.
            None => {
                intent_hash.update(b"worth.query.document-history.envelope.intent.v1");
                intent_hash.update(envelope.intent.identity());
            }
            Some(correction) => {
                intent_hash.update(b"worth.query.document-history.envelope.correction-intent.v1");
                intent_hash.update(envelope.intent.identity());
                intent_hash.update((correction.binding.len() as u64).to_be_bytes());
                intent_hash.update(correction.binding.as_bytes());
                intent_hash.update(correction.source.ordinal().to_be_bytes());
                intent_hash.update((correction.member.len() as u64).to_be_bytes());
                intent_hash.update(correction.member.as_bytes());
            }
        }
        let meaning = <WorthQueryApplicationIdempotencyBinding as IdempotencyBindingInternalPort>::from_identities(
            key_hash.finalize().into(),
            intent_hash.finalize().into(),
        );
        let head = runtime
            .on_branch(self.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        let observed =
            WorthQueryProductBranchReadIdentity::from_observation(head.product().observation());
        drop(head);
        let execution = phase
            .checked_execution_request_for(runtime.inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        let mut selected = self.selection.lock();
        match selected.journal.lookup(meaning) {
            AttemptLookup::Completed(node) => {
                return Ok(WorthQueryDocumentHistoryRecordPreparation::AlreadyRecorded(
                    selected
                        .journal
                        .step(node)
                        .expect("completed envelope binds a completed node")
                        .clone(),
                ))
            }
            AttemptLookup::Drift => return Err(Denial::IntentDrift),
            AttemptLookup::Reserved => return Err(Denial::AttemptInFlight),
            AttemptLookup::Unseen => {}
        }
        if selected.expected != observed {
            return Err(Denial::HeadDiverged);
        }
        let record = selected
            .journal
            .reserve(&execution, meaning, label, step_metadata)?;
        Ok(WorthQueryDocumentHistoryRecordPreparation::Reserved(
            WorthQueryDocumentHistoryReservation {
                operation_scope: None,
                record,
                expected: selected.expected.clone(),
                meaning,
            },
        ))
    }
}

/// A document's navigation lineage: the journal's O(log) source and basis
/// indexes, read under the document's selection lock.
struct DocumentNavigationLineage {
    selection: super::super::selection::WeakHistorySelectionCell,
}

impl crate::domain_computation::primary_graph::product_operation::WorthQueryCorrectionNavigationLineage
    for DocumentNavigationLineage
{
    fn restores_descendant_of(
        &self,
        selected: worth_relational::facade::history::CommitId,
        source: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) -> bool {
        // A closed document vouches for nothing.
        self.selection
            .upgrade()
            .is_some_and(|selection| selection.lock().journal.restores_descendant_of(selected, source))
    }
}
