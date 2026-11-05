use crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity;
use crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity;

impl<Query> super::super::WorthQueryObservedSource<Query> {
    /// Runtime-local source commitment retained from the immutable basis.
    /// Branch occurrence remains part of ordinary output lineage; checkpoint
    /// recovery uses the separate portable checkpoint identity.
    pub fn idempotency_identity(&self) -> WorthQueryRuntimeSourceIdentity {
        self.source_meaning.runtime_idempotency_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity(
        &self,
    ) -> WorthQueryCheckpointSourceIdentity {
        self.source_meaning.checkpoint_identity()
    }
}

mod court_doors_for_test;
