//! Read-origin tree disposal is distinct from required source completion.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial as Denial;
impl<S, P, R> WorthQueryApplicationProgramOutputHandle<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S>
        + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(in crate::application_entry) fn close_read_request(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), Denial> {
        self.drain_read_request(phase, application, branch)?;
        self.continuation = None;
        self.root = None;
        self.root_settlement = None;
        self.pending_root_authority = None;
        self.pending_history_delivery = None;
        self.complete = true;
        Ok(())
    }

    /// Drain actual read owners while retaining original permission subjects.
    /// Selection retries must freshly authorize every completed tree member.
    pub(in crate::application_entry) fn drain_read_request(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), Denial> {
        if self.source_receipt.is_some()
            || self.source_preparation.is_some()
            || self.source_binding.is_some()
        {
            return Err(Denial::ForeignDemand);
        }
        self.drain_original_interests(phase, application, branch)
    }

    /// Dispose only the required tree owned by a retiring history candidate.
    /// Its source remains unfulfilled; this never runs a producer or source.
    pub(in crate::application_entry) fn close_group_original(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        aborting: &worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial> {
        use crate::application_entry::WorthQueryOperationCloseDenial as Close;
        aborting
            .validate_candidate_cleanup(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                application.runtime(),
                branch,
            )
            .map_err(Close::HistoryGroup)?;
        if !self
            .source_receipt
            .as_ref()
            .is_some_and(|receipt| receipt.product_branch() == branch)
            || self.source_binding.is_none()
            || self.source_preparation.is_none()
        {
            return Err(Close::OutputRead(Denial::ForeignDemand));
        }
        self.drain_original_interests(phase, application, branch)
            .map_err(Close::OutputRead)?;
        self.continuation = None;
        self.root = None;
        self.pending_root_authority = None;
        self.pending_root_fulfillment = None;
        self.pending_history_delivery = None;
        // Original source/preparation/read pins remain in this closing owner
        // until its independent managed cleanup has also succeeded.
        Ok(())
    }

    fn drain_original_interests(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), Denial> {
        if let Some(children) = self.continuation.as_mut() {
            children.close_read_request(phase, application, branch)?;
        }
        if let Some(root) = self.root.as_mut() {
            root.close_read_request(phase, application, branch)?;
        }
        Ok(())
    }
}
