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
        if self.source_receipt.is_some()
            || self.source_preparation.is_some()
            || self.source_validator.is_some()
        {
            return Err(Denial::ForeignDemand);
        }
        if let Some(children) = self.continuation.as_mut() {
            children.close_read_request(phase, application, branch)?;
        }
        if let Some(root) = self.root.as_mut() {
            root.close_read_request(phase, application, branch)?;
        }
        self.continuation = None;
        self.root = None;
        self.root_settlement = None;
        self.pending_root_authority = None;
        self.complete = true;
        Ok(())
    }
}
