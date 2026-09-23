use super::WorthQueryObservedSource;

/// Owner-issued evidence for the membership of one completed query result set.
/// Unlike a row source, this also retains observations that selected no row.
pub struct WorthQueryObservedResultSet<Query> {
    pub(super) source: WorthQueryObservedSource<Query>,
}

impl<Query> WorthQueryObservedResultSet<Query> {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        source: WorthQueryObservedSource<Query>,
    ) -> Self {
        Self { source }
    }

    pub(in crate::domain_computation) fn idempotency_identity(&self) -> [u8; 32] {
        self.source.idempotency_identity()
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn selection_for_test(
        &self,
    ) -> &super::WorthQueryObservedRootSelection {
        self.source
            .footprint
            .root_selection
            .as_deref()
            .expect("result-set evidence retains native selection facts")
    }
}
