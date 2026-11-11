//! One original index-state observation carries catalog and actual cache owners.
use super::*;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionArray, ExecutionRequest};

pub(crate) struct FrozenCheckpointIndexes {
    catalog: CapturedIndexDefinitionCatalog,
    generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    generation_frontier: u64,
    bases: basis_selection::FrozenBasisSelections,
    deferred: Option<Arc<cold_checkpoint::DeferredIndexImage>>,
}
pub(crate) struct OriginalCheckpointIndexes {
    pub(crate) catalog: CapturedIndexDefinitionCatalog,
    generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    pub(crate) generation_frontier: u64,
    bases: basis_selection::OriginalCheckpointBases,
    _decoded: Option<
        crate::durability::derived_index_artifacts::captured_decode::CapturedCheckpointGenerations,
    >,
}
impl OriginalCheckpointIndexes {
    pub(crate) fn generations(&self) -> &[Arc<DerivedIndexGeneration>] {
        &self.generations
    }
    /// Exact-root selections over `generations`, including a never-admitted image's.
    pub(crate) fn bases(&self) -> &[crate::indexes::data::IndexBasisCheckpoint] {
        self.bases.rows()
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
            bases: state.bases.freeze_for_request(request)?,
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
        let decoded = match self.deferred {
            Some(image) => Some(
                image
                    .capture_generations(request)
                    .map_err(crate::durability::checkpoint_history::capture::capture_stop)?,
            ),
            None => None,
        };
        if decoded.is_none() && self.bases.rostered(&self.generations) {
            let bases = self.bases.resolve(&[], &self.generations, request)?;
            return Ok(OriginalCheckpointIndexes {
                catalog: self.catalog,
                generations: self.generations,
                generation_frontier: self.generation_frontier,
                bases,
                _decoded: None,
            });
        }
        // The emitted roster is the catalog, a never-admitted image, and every
        // payload a live selection still binds after its publication retired.
        let image = decoded
            .iter()
            .flat_map(|decoded| decoded.generations.iter());
        let mut identities = worth_execution::ExecutionOrderedMap::default();
        for generation in self
            .bases
            .bound()
            .chain(image)
            .chain(self.generations.iter())
        {
            crate::execution::check_request(request).map_err(Denial::Request)?;
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
        let generations = roster
            .seal()
            .map_err(crate::durability::checkpoint_history::capture::array_capture_stop)?;
        // Reclose before any image demand must not drop what the image carried.
        let carried = decoded
            .as_ref()
            .map_or(&[][..], |decoded| decoded.bases.as_slice());
        let bases = self.bases.resolve(carried, &generations, request)?;
        Ok(OriginalCheckpointIndexes {
            catalog: self.catalog,
            generations,
            generation_frontier: self.generation_frontier,
            bases,
            _decoded: decoded,
        })
    }
}
