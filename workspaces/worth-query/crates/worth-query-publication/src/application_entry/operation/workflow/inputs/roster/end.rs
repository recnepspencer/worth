//! The end of the input set cannot manufacture operation or completion custody.
use super::*;
use std::convert::Infallible;
impl sealed::List for WorthQueryWorkflowInputEnd {}
impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>> WorthQueryWorkflowInputList<S, P>
    for WorthQueryWorkflowInputEnd
{
    type Operation = Infallible;
    type Completed = Infallible;
    type Terminal = Infallible;
    fn capture(
        &self,
        _: &mut super::super::capture::WorthQueryWorkflowInputCapture<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryWorkflowInputDenial> {
        Ok(())
    }
    fn registration_denial(
        &self,
        _: &str,
        _: Option<
            &worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity,
        >,
    ) -> Option<WorthQueryWorkflowInputDenial> {
        None
    }
    fn select<Spec: ApplicationWorkflowSpec<Schema = S>>(
        &mut self,
        _: &WorthQueryProgramApplicationRuntime<S, P>,
        _: WorthQueryWorkflowVocabulary<'_, S, Spec>,
        _: &RequiredWorkflowOperation,
    ) -> Result<Infallible, WorthQueryWorkflowInputDenial> {
        Err(WorthQueryWorkflowInputDenial::NotSupplied)
    }
    fn advance(
        owner: Infallible,
        _: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryWorkflowOperationAdvance<Infallible, Infallible, Infallible> {
        match owner {}
    }
    fn required(owner: &Infallible) -> &RequiredWorkflowOperation {
        match *owner {}
    }
    fn completed_required(owner: &Infallible) -> &RequiredWorkflowOperation {
        match *owner {}
    }
    fn accept_completed<Spec, O, Input, Scope>(
        owner: Infallible,
        _: WorthQueryWorkflowAdvanceRequest<'_, '_, '_, S, Spec, O, Input, Scope>,
    ) -> Result<
        WorthQueryWorkflowAcceptance<Infallible>,
        (Infallible, WorthQueryWorkflowOperationOwnerAcceptanceDenial),
    >
    where
        S: 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        O: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        match owner {}
    }
    fn close(
        owner: Infallible,
        _: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Infallible, WorthQueryOperationCloseDenial)> {
        match owner {}
    }
    fn close_completed(
        owner: Infallible,
        _: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Infallible, WorthQueryOperationCloseDenial)> {
        match owner {}
    }
    fn close_terminal(
        owner: Infallible,
        _: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Infallible, WorthQueryOperationCloseDenial)> {
        match owner {}
    }
}
