//! Bind checkpoint projection to the installed read handles.
use super::RelationalRuntime;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::CheckpointProjectionContext;

impl RelationalRuntime {
    pub(crate) fn checkpoint_projection_context(
        &self,
        lane: &ReconstructionLane<'_>,
    ) -> CheckpointProjectionContext {
        CheckpointProjectionContext::from_read_handles(
            lane,
            self.runtime_instance_id(),
            self.current_version_id(),
            self.services.instrumentation.clone(),
            self.services.symbols.clone(),
        )
    }
}
