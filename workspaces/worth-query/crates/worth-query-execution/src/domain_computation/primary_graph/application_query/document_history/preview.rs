//! Retained truth and the current document READ have distinct custody and checks.
use super::*;
use crate::domain_computation::primary_graph::{WorthQueryHistoryNodeId, WorthQueryHistoryPreview};

mod admission;
pub use admission::WorthQueryHistoryPreviewReadAdmission;

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn open_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<WorthQueryHistoryPreview, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed = read.complete_metadata_read(self, phase, document, access)?;
        let preview = document
            .selection
            .lock()
            .journal
            .preview(node, document.model.clone())?;
        completed.confirm(document, preview)
    }

    pub fn prepare_document_history_preview_read<'phase, 'scope, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        preview: &WorthQueryHistoryPreview,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryPreviewReadAdmission<'phase, 'scope>, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let query = read.plan.query.identity().clone();
        let binding = read
            .plan
            .query
            .binding_observation()
            .cloned()
            .ok_or(Denial::InstalledBindingMismatch)?;
        let parameters = *read.plan.parameters.identity();
        let principal = read.plan.principal.principal_entity_id();
        let completed = read.complete_metadata_read(self, phase, document, access)?;
        if !document
            .selection
            .lock()
            .journal
            .owns_node(preview.step.id())
            || preview.model != document.model
        {
            return Err(Denial::ForeignHistoryNode);
        }
        Ok(WorthQueryHistoryPreviewReadAdmission::new(
            completed, preview, query, binding, parameters, principal,
        ))
    }

    pub fn read_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        admission: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<Query<S, H>, Rows<S, H>>, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        admission.validate_plan(phase, &read.plan)?;
        if read.binding.query().binding_observation() != read.plan.query.binding_observation() {
            return Err(Denial::InstalledBindingMismatch);
        }
        let (historical, result) = read.execute(self, phase)?;
        if historical.model() != admission.model
            || historical.identity() != *admission.step.endpoint()
        {
            return Err(Denial::ReceiptMismatch);
        }
        if !document
            .selection
            .lock()
            .journal
            .owns_node(admission.step.id())
            || document.model != admission.model
        {
            return Err(Denial::ForeignHistoryNode);
        }
        // Historical truth must not be compared to its branch's current head.
        // Only the separately captured current document is the delivery guard.
        admission.completed.confirm(document, result)
    }
}
