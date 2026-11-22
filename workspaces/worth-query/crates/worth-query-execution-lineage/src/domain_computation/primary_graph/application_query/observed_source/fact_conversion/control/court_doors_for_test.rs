//! Sealed test doors over this module's private members.
use super::*;

impl<'request, 'authority> SourceFactExpansionControl<'request, 'authority> {
    #[doc(hidden)]
    pub fn in_request_for_test(
        execution: ExecutionRequest<'request, 'authority>,
        request: Option<&'request WorthQueryRequestScope>,
    ) -> Self {
        Self::in_request(execution, request)
    }
}
