//! Redo hints are metadata under the same actual READ and final selection guard.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryPageCursor, WorthQueryHistoryPageDenial, WorthQueryHistoryPageLimits,
    WorthQueryHistoryRedoChoices,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn read_document_history_redo_choices<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryRedoChoices, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed = read.complete_metadata_read(self, phase, document, access)?;
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(Denial::Stopped)?;
        let context = document.history_description_context(access);
        let (choices, generation) = {
            let selected = document.selection.lock();
            (
                selected
                    .journal
                    .redo_choices(context, execution, limits, after)
                    .map_err(Denial::Paging)?,
                selected.journal.view_generation(),
            )
        };
        if let Some(pause) = &document.page_pause {
            pause.pause();
        }
        let choices = execution
            .run_native(|_| choices)
            .map_err(|cause| Denial::Paging(WorthQueryHistoryPageDenial::Request(cause)))?
            .0;
        completed.confirm_generation(document, generation, choices)
    }
}
