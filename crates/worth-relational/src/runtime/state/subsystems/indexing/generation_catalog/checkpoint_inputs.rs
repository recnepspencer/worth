//! A fixed roster of actual generation owners in the legacy canonical order.
use super::*;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};
impl GenerationCatalog {
    pub(in crate::runtime::state::subsystems::indexing) fn snapshot_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        ExecutionArray<Arc<DerivedIndexGeneration>>,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        use crate::durability::checkpoint_history::capture::{
            array_capture_stop, capture_stop, map_stop,
        };
        use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
        let mut ordered = worth_execution::ExecutionOrderedMap::default();
        for generation in self.entries.values() {
            crate::execution::check_request(request).map_err(Denial::Request)?;
            ordered
                .insert_cloned_native_for_request(
                    &(generation.index_id, generation.generation_id),
                    generation,
                    *request,
                )
                .map_err(|cause| capture_stop(map_stop(cause)))?;
        }
        let mut roster = ExecutionRequestArrayBuilder::allocate(*request, ordered.len())
            .map_err(array_capture_stop)?;
        for (_, generation) in ordered.iter() {
            request
                .consult()
                .map_err(|cause| Denial::Request(cause.into()))?;
            roster
                .push(generation.clone())
                .map_err(array_capture_stop)?;
        }
        roster.seal().map_err(array_capture_stop)
    }
}
