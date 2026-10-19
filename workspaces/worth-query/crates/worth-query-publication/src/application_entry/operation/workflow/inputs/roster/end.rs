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
    fn contains(&self, _: &str) -> bool {
        false
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
