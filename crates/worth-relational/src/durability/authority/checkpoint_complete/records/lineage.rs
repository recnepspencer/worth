//! Original Create targets bind positionally to canonical entity patch records.
use crate::durability::data::RelationalNativeCheckpointCaptureDenial as Denial;
use crate::history::data::CanonicalCommitEnvelope;
use crate::lineage::data::LineageEventKind;
use crate::publication::patch::data::RecordStructuralChange;
use crate::storage::overlay::{CurrentPartitionAccess, WorkingState};
use crate::transactions::data::RecordRef;
use worth_execution::ExecutionRequest;

pub(super) fn install(
    execution: &ExecutionRequest<'_, '_>,
    state: &mut WorkingState,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Denial> {
    let mut creates = envelope
        .lineage_events()
        .iter()
        .filter(|event| event.kind() == LineageEventKind::Create);
    for patch in &envelope.patch.authoritative_record_patches {
        let RecordRef::Entity(entity) = patch.target else {
            continue;
        };
        if patch.structural_change != RecordStructuralChange::Created {
            continue;
        }
        let event = creates.next().ok_or(Denial::OriginalAuthorityUnavailable)?;
        if event.commit() != &envelope.commit
            || event.targets().len() != 1
            || !event.sources().is_empty()
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let lineage = event.targets()[0];
        let previous = state
            .get_current_partition(entity.partition_id)
            .and_then(|partition| partition.entity_arena().get(&entity))
            .ok_or(Denial::OriginalAuthorityUnavailable)?
            .extra()
            .lineage_id;
        if let Some(previous) = previous {
            if previous != lineage {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            continue;
        }
        state
            .get_partition_mut_for_request(entity.partition_id, execution)
            .map_err(Denial::Request)?
            .entity_arena
            .install_lineage_id(execution, entity.slot_index(), lineage)
            .map_err(|cause| match cause {
                crate::storage::substrate::ExtraLineageDenial::Request(cause) => {
                    Denial::Request(cause)
                }
                crate::storage::substrate::ExtraLineageDenial::Missing(_) => {
                    Denial::OriginalAuthorityUnavailable
                }
            })?;
    }
    if creates.next().is_some() {
        return Err(Denial::OriginalAuthorityUnavailable);
    }
    Ok(())
}
