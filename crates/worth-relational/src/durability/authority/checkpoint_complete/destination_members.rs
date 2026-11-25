//! Carry eligibility comes from the destination's actual selected membership.
use super::Denial;
use crate::durability::{
    data::{RelationalCheckpointArtifactPolicy, RelationalCheckpointMember},
    medium::RelationalDurableMemberLocator,
};
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) fn require(
    placement: &RelationalCheckpointArtifactPolicy<'_>,
    members: &ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    let RelationalCheckpointArtifactPolicy::ExternalMembers {
        current_members, ..
    } = placement;
    let mut selected = ExecutionOrderedMap::default();
    for locator in *current_members {
        selected
            .insert_cloned_native_for_request(locator, &(), *request)
            .map_err(|cause| {
                crate::durability::checkpoint_history::capture::capture_stop(
                    crate::durability::checkpoint_history::capture::map_stop(cause),
                )
            })?;
    }
    for member in members.values() {
        crate::execution::check_request(request).map_err(Denial::Request)?;
        if let RelationalCheckpointMember::Carried { locator } = member
            && !selected.contains_key(locator)
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
    }
    Ok(())
}
