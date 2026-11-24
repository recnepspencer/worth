//! Carry eligibility comes from the destination's actual selected membership.
use crate::durability::data::RelationalCheckpointArtifactPolicy;
use contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember;

pub(super) fn require(
    placement: &RelationalCheckpointArtifactPolicy<'_>,
    members: &ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    match placement {
        RelationalCheckpointArtifactPolicy::Embedded => {
            if members
                .values()
                .any(|member| matches!(member, RelationalCheckpointMember::Carried { .. }))
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
        }
        RelationalCheckpointArtifactPolicy::ExternalMembers {
            current_members, ..
        } => {
            let mut selected = ExecutionOrderedMap::default();
            for locator in *current_members {
                selected
                    .insert_cloned_native_for_request(locator, &(), *request)
                    .map_err(|cause| {
                        contracts::durability::data::native_checkpoint_capture_denial::capture_stop(
                            contracts::durability::data::recovery_stop::map_stop(cause),
                        )
                    })?;
            }
            for member in members.values() {
                contracts::execution::request_custody::check_request(request)
                    .map_err(Denial::Request)?;
                if let RelationalCheckpointMember::Carried { locator } = member
                    && !selected.contains_key(locator)
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            }
        }
    }
    Ok(())
}
