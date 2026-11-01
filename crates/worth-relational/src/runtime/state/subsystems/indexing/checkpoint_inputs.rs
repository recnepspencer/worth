//! One original index-state observation carries catalog and actual cache owners.
use super::*;
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionArray, ExecutionRequest};

pub(crate) struct OriginalCheckpointIndexes {
    pub(crate) catalog: CapturedIndexDefinitionCatalog,
    pub(crate) generations: ExecutionArray<Arc<DerivedIndexGeneration>>,
    pub(crate) generation_frontier: u64,
}
impl IndexingSubsystem {
    pub(crate) fn checkpoint_inputs_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalCheckpointIndexes, Denial> {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        let state = self.state.read();
        // A deferred image is not an admitted cache roster. Do not demand,
        // rebuild, omit or replace it while preparing an original Complete cut.
        if state.cold_checkpoint.is_some() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let generations = state.generations.snapshot_for_request(request)?;
        Ok(OriginalCheckpointIndexes {
            catalog: CapturedIndexDefinitionCatalog {
                catalog: state.catalog.clone(),
                coverage: state.catalog.coverage(),
                next_index_id: state.next_index_id,
            },
            generations,
            generation_frontier: state.generation_frontier(),
        })
    }
}
