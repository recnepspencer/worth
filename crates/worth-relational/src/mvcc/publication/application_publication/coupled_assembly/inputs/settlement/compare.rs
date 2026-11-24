//! Compare exact full-generation role payloads across two candidate images.
use crate::execution::request_stage::checkpoint;
use crate::identity::data::{EntityId, KindId};
use crate::mvcc::publication::application_publication::custody;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RoleSignature<'a> {
    pub(super) role: &'a str,
    pub(super) entity_name: &'a str,
    pub(super) kind: KindId,
    pub(super) entity: EntityId,
    pub(super) retiring: bool,
}

pub(super) fn payloads_equal(
    execution: &ExecutionRequest<'_, '_>,
    old_root: &impl CurrentPartitionAccess,
    new_root: &impl CurrentPartitionAccess,
    roles: &[RoleSignature<'_>],
) -> Result<bool, TransactionCommitError> {
    execution
        .run_native_with_checkpoint(|context| {
            for role in roles {
                checkpoint(context, 1).map_err(custody::execution_stop)?;
                if !entity_payload_equal(old_root, new_root, role.entity) {
                    return Ok::<_, TransactionCommitError>(false);
                }
            }
            Ok(true)
        })
        .map_err(custody::execution_stop)?
        .0
}

fn entity_payload_equal(
    old_root: &impl CurrentPartitionAccess,
    new_root: &impl CurrentPartitionAccess,
    entity: EntityId,
) -> bool {
    let slot = entity.slot_index();
    let old_slot = old_root
        .get_current_partition(entity.partition_id)
        .and_then(|p| p.entity_arena().get_slot(slot));
    let new_slot = new_root
        .get_current_partition(entity.partition_id)
        .and_then(|p| p.entity_arena().get_slot(slot));
    let (Some(old_slot), Some(new_slot)) = (old_slot, new_slot) else {
        return false;
    };
    if old_slot.record_id() != entity
        || new_slot.record_id() != entity
        || !old_slot.is_live()
        || !new_slot.is_live()
        || old_slot.kind_id() != new_slot.kind_id()
    {
        return false;
    }
    old_slot.extra().authoritative_aspect_state() == new_slot.extra().authoritative_aspect_state()
}
