//! Native close consumes only lawful original custody and returns it on refusal.
use super::*;
impl<I: ApplicationMutationIntent<WorthUiApplicationSchema> + 'static, A> State<I, A> {
    pub(super) fn close(self, request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>) -> Result<Option<WorthQueryClosedOperation>, (Self, WorthQueryOperationCloseDenial)> {
        match self {
            Self::Original(_) | Self::Capture(_) | Self::Description(_) => Ok(None),
            Self::Pending(operation) => operation.close(request).map(Some).map_err(|(operation, denial)| (Self::Pending(operation), denial)),
            Self::Completed(operation) => operation.close(request).map(Some).map_err(|(operation, denial)| (Self::Completed(operation), denial)),
            Self::Terminal(operation) => operation.close(request).map(Some).map_err(|(operation, denial)| (Self::Terminal(operation), denial)),
        }
    }
}
impl NativeCustody {
    pub(in crate::product_projection::status_mutation) fn close(self, request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>) -> Result<Option<WorthQueryClosedOperation>, (Self, WorthQueryOperationCloseDenial)> {
        match self {
            Self::Source(state) => state.close(request).map_err(|(state, denial)| (Self::Source(state), denial)),
            Self::Action(state) => state.close(request).map_err(|(state, denial)| (Self::Action(state), denial)),
        }
    }
}
