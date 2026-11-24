use super::{prepare, CompleteRelationalNativeCheckpoint, Denial};
use crate::durability::authority::runtime_access::DurabilityAuthority;

impl DurabilityAuthority<'_> {
    pub(crate) fn complete_native_checkpoint_with_policy(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        placement: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CompleteRelationalNativeCheckpoint, Denial> {
        prepare::prepare(self.runtime, execution, placement)
    }
}
