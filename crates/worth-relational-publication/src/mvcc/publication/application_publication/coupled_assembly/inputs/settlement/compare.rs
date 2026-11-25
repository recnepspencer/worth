//! Compare exact full-generation role payloads across two candidate images.
use crate::mvcc::publication::application_publication::custody;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleSignature<'a> {
    pub role: &'a str,
    pub entity_name: &'a str,
    pub kind: KindId,
    pub entity: EntityId,
    pub retiring: bool,
}

pub fn payloads_equal(
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
