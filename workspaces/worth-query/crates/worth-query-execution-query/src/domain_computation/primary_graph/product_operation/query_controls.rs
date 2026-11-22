use std::num::NonZeroUsize;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

/// Result cardinality and request scope for an already selected product. These controls
/// cannot select a second basis or change the operation's retained occurrence.
pub struct WorthQueryProductQueryControls<'request> {
    pub(crate) maximum_results: NonZeroUsize,
    pub(crate) request: &'request WorthQueryRequestScope,
}

impl<'request> WorthQueryProductQueryControls<'request> {
    pub fn new(maximum_results: NonZeroUsize, request: &'request WorthQueryRequestScope) -> Self {
        Self {
            maximum_results,
            request,
        }
    }
}

/// Crate-boundary port for `WorthQueryProductQueryControls` consumed by
/// `worth-query-execution`. Never exported on `worth-query-execution::facade`
/// or Host.
pub trait ProductQueryControlsInternalPort<'request> {
    fn maximum_results(&self) -> NonZeroUsize;
    fn request(&self) -> &'request WorthQueryRequestScope;
}

impl<'request> ProductQueryControlsInternalPort<'request>
    for WorthQueryProductQueryControls<'request>
{
    fn maximum_results(&self) -> NonZeroUsize {
        self.maximum_results
    }

    fn request(&self) -> &'request WorthQueryRequestScope {
        self.request
    }
}
