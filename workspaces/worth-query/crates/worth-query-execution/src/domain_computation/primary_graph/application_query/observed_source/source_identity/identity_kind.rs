#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRuntimeSourceIdentity([u8; 32]);

impl WorthQueryRuntimeSourceIdentity {
    pub(in crate::domain_computation::primary_graph) const fn new(identity: [u8; 32]) -> Self {
        Self(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCheckpointSourceIdentity(
    [u8; 32],
);

impl WorthQueryCheckpointSourceIdentity {
    pub(in crate::domain_computation::primary_graph) const fn new(identity: [u8; 32]) -> Self {
        Self(identity)
    }

    pub(in crate::domain_computation::primary_graph) const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

impl<Query> super::super::WorthQueryObservedSource<Query> {
    /// Runtime-local source commitment retained from the immutable basis.
    /// Branch occurrence remains part of ordinary output lineage; checkpoint
    /// recovery uses the separate portable checkpoint identity.
    pub(in crate::domain_computation::primary_graph) fn idempotency_identity(
        &self,
    ) -> WorthQueryRuntimeSourceIdentity {
        self.source_meaning.runtime_idempotency_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity(
        &self,
    ) -> WorthQueryCheckpointSourceIdentity {
        self.source_meaning.checkpoint_identity()
    }
}
