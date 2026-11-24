use super::{capture, prepare, CompleteRelationalNativeCheckpoint, Denial};
use crate::durability::authority::runtime_access::DurabilityAuthority;
use worth_relational_durability::internal_port::durability::data::checkpoint_member::RelationalCheckpointArtifactPolicyOwnerPort as _;

impl DurabilityAuthority<'_> {
    pub(crate) fn complete_native_checkpoint_with_policy(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        placement: crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<CompleteRelationalNativeCheckpoint, Denial> {
        crate::execution::check_request(execution).map_err(Denial::Request)?;
        if let (Some(actual), Some(selected)) = (
            self.runtime.durability.cold_source(),
            placement.publication_source(),
        ) && !std::sync::Arc::ptr_eq(&actual, &selected)
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let frozen =
            capture::freeze::capture(self.runtime, execution, &placement)?.resolve(execution)?;
        prepare::prepare(
            frozen,
            self.runtime.durability.checkpoint_selection(),
            self.runtime.durability.medium(),
            execution,
            placement,
        )
    }
}
