use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Result cardinality and request scope for an already selected product. These controls
/// cannot select a second basis or change the operation's retained occurrence.
pub struct WorthQueryProductQueryControls<'request> {
    pub maximum_results: NonZeroUsize,
    pub request: &'request WorthQueryRequestScope,
}

impl<'request> WorthQueryProductQueryControls<'request> {
    pub fn new(maximum_results: NonZeroUsize, request: &'request WorthQueryRequestScope) -> Self {
        Self {
            maximum_results,
            request,
        }
    }

    pub fn into_query_parts(
        self,
        _seal: crate::ports::WorthQueryQueryCrateSeal,
    ) -> (NonZeroUsize, &'request WorthQueryRequestScope) {
        (self.maximum_results, self.request)
    }
}
