//! One original index-state observation carries catalog and actual cache owners.
use super::*;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionArray, ExecutionRequest};

pub(crate) struct FrozenCheckpointIndexes {
    catalog: CapturedIndexDefinitionCatalog,
    generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    generation_frontier: u64,
    deferred: Option<Arc<cold_checkpoint::DeferredIndexImage>>,
}
pub(crate) struct OriginalCheckpointIndexes {
    pub(crate) catalog: CapturedIndexDefinitionCatalog,
    generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    pub(crate) generation_frontier: u64,
    _decoded: Option<
        crate::durability::derived_index_artifacts::captured_decode::CapturedCheckpointGenerations,
    >,
}
impl OriginalCheckpointIndexes {
    pub(crate) fn generations(&self) -> &[Arc<DerivedIndexGeneration>] {
        &self.generations
    }
}
impl IndexingSubsystem {
    pub(crate) fn checkpoint_inputs_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<FrozenCheckpointIndexes, Denial> {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        let state = self.state.read();
        let generations = state.generations.snapshot_for_request(request)?;
        Ok(FrozenCheckpointIndexes {
            catalog: CapturedIndexDefinitionCatalog {
                catalog: state.catalog.clone(),
                coverage: state.catalog.coverage(),
                next_index_id: state.next_index_id,
            },
            generations,
            generation_frontier: state.generation_frontier(),
            deferred: state.cold_checkpoint.clone(),
        })
    }
}

impl FrozenCheckpointIndexes {
    pub(crate) fn resolve_for_request(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalCheckpointIndexes, Denial> {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        let Some(image) = self.deferred else {
            return Ok(OriginalCheckpointIndexes {
                catalog: self.catalog,
                generations: self.generations,
                generation_frontier: self.generation_frontier,
                _decoded: None,
            });
        };
        let decoded = image
            .capture_generations(request)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        let mut identities = worth_execution::ExecutionOrderedMap::default();
        for generation in decoded.generations.iter().chain(self.generations.iter()) {
            identities
                .insert_cloned_native_for_request(&generation.generation_id, generation, *request)
                .map_err(|cause| {
                    crate::durability::checkpoint_history::capture::capture_stop(
                        crate::durability::checkpoint_history::capture::map_stop(cause),
                    )
                })?;
        }
        let mut ordered = worth_execution::ExecutionOrderedMap::default();
        for (_, generation) in identities.iter() {
            ordered
                .insert_cloned_native_for_request(
                    &(generation.index_id, generation.generation_id),
                    generation,
                    *request,
                )
                .map_err(|cause| {
                    crate::durability::checkpoint_history::capture::capture_stop(
                        crate::durability::checkpoint_history::capture::map_stop(cause),
                    )
                })?;
        }
        let mut roster =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, ordered.len())
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        for (_, generation) in ordered.iter() {
            if generation.generation_id.0 == 0
                || generation.generation_id.0 >= self.generation_frontier
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            roster
                .push(generation.clone())
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        }
        Ok(OriginalCheckpointIndexes {
            catalog: self.catalog,
            generations: roster
                .seal()
                .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?,
            generation_frontier: self.generation_frontier,
            _decoded: Some(decoded),
        })
    }
}
