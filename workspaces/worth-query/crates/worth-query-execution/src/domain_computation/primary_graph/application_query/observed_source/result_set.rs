use super::WorthQueryObservedSource;

/// Owner-issued evidence for the membership of one completed query result set.
/// Unlike a row source, this also retains observations that selected no row.
pub struct WorthQueryObservedResultSet<Query> {
    pub(super) source: WorthQueryObservedSource<Query>,
}

// Retain the same completed-set proof for a retry; cloning does not issue a
// row proof, reobserve the graph or materialize its facts.
impl<Query> Clone for WorthQueryObservedResultSet<Query> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
        }
    }
}

impl<Query> WorthQueryObservedResultSet<Query> {
    pub(in crate::domain_computation::primary_graph::application_query) fn new(
        source: WorthQueryObservedSource<Query>,
    ) -> Self {
        Self { source }
    }

    pub(in crate::domain_computation) fn idempotency_identity(&self) -> [u8; 32] {
        self.source.idempotency_identity().bytes()
    }

    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &WorthQueryObservedSource<Query> {
        &self.source
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn selection_for_test(
        &self,
    ) -> &super::WorthQueryObservedRootSelection {
        self.source
            .source_meaning
            .footprint()
            .root_selection
            .as_deref()
            .expect("result-set evidence retains native selection facts")
    }
}
