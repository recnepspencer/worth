//! One envelope reserves one history record before any candidate exists.
use super::super::{
    journal::AttemptLookup, WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
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
        }
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
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
        intent_hash.update(b"worth.query.document-history.envelope.intent.v1");
        intent_hash.update(envelope.intent.identity());
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
