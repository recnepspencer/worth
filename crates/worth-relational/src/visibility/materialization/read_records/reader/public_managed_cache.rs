//! Conservative cache access resolves the genuine selected image without a Source program.
use super::VisibilityReadContext;
use crate::{
    mvcc::ManagedIndexPublicCache, snapshots::data::SnapshotHandle,
    validation::candidate_source::ManagedIndexSourcePreparationDenial as Denial,
};
use std::{any::Any, sync::Arc};
use worth_execution::ExecutionRequest;

impl VisibilityReadContext<'_> {
    /// Retain opaque material only; the caller separately admits Source READ/build purpose.
    pub fn retain_public_managed_index_cache(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        payload: &Arc<dyn Any + Send + Sync>,
    ) -> Result<ManagedIndexPublicCache, Denial> {
        let (selected, memory) = self.managed_source_position(execution, handle)?;
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime, handle)
                .ok_or(Denial::RootUnavailable)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        basis
            .root()
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?
            .retain_public_cache(execution, selected, generation, payload, memory)
    }

    /// Reborrow only the cache attached to this actual selected Native image.
    pub fn public_managed_index_cache_payload(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        handle: &SnapshotHandle,
        cache: &ManagedIndexPublicCache,
    ) -> Result<Arc<dyn Any + Send + Sync>, Denial> {
        let (selected, _memory) = self.managed_source_position(execution, handle)?;
        let basis =
            crate::visibility::snapshot_states::resolve_snapshot_basis(self.runtime, handle)
                .ok_or(Denial::RootUnavailable)?;
        let publication = self.runtime.publication_binding();
        let epoch = publication
            .companion_epoch()
            .map_err(Denial::Registration)?;
        let (generation, _) = epoch
            .active()
            .map_err(Denial::Registration)?
            .ok_or(Denial::MissingParticipant)?;
        basis
            .root()
            .managed_source_image
            .get()
            .ok_or(Denial::RootUnavailable)?
            .public_cache_payload(&selected, generation, cache)
    }
}
