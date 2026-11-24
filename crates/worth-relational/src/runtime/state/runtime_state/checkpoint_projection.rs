//! Bind checkpoint projection to the installed read handles.
use super::RelationalRuntime;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::CheckpointProjectionContext;

impl RelationalRuntime {
    pub(crate) fn checkpoint_projection_context(
        &self,
        lane: &ReconstructionLane<'_>,
    ) -> CheckpointProjectionContext {
        CheckpointProjectionContext::from_inputs(lane, self.into())
    }
}

impl<'a> From<&'a RelationalRuntime>
    for &'a crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection
{
    fn from(runtime: &'a RelationalRuntime) -> Self {
        runtime.durability.checkpoint_selection()
    }
}

impl<'a> From<&'a RelationalRuntime>
    for &'a crate::runtime::state::subsystems::indexing::IndexingSubsystem
{
    fn from(runtime: &'a RelationalRuntime) -> Self {
        &runtime.indexes
    }
}
