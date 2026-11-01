//! A fresh metadata READ precedes cursor/range work; a final guard closes assembly.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHistoryPage, WorthQueryHistoryPageCursor, WorthQueryHistoryPageDenial,
    WorthQueryHistoryPageLimits,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn read_document_history_page<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed = read.complete_metadata_read(self, phase, document, access)?;
        let execution = phase
            .checked_execution_request_for(&self.product_runtime)
            .map_err(Denial::Stopped)?;
        let (page, generation) = {
            let selected = document.selection.lock();
            (
                selected
                    .journal
                    .page(execution, limits, after)
                    .map_err(Denial::Paging)?,
                selected.journal.view_generation(),
            )
        };
        if let Some(pause) = &document.page_pause {
            pause.pause();
        }
        let page = execution
            .run_native(|_| page)
            .map_err(|cause| Denial::Paging(WorthQueryHistoryPageDenial::Request(cause)))?
            .0;
        completed.confirm_generation(document, generation, page)
    }
}
