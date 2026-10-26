use worth_foundational::facade::PortablePatchReadmissionPurpose;

use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::ensure_relation_target_is_current;
use crate::authority::mutation::MutationWorkspace;
use crate::storage::overlay::CurrentPartitionAccess;
use crate::storage::substrate::RelationExtra;
use crate::transactions::data::{
    ApplyRelationAspectPatchIntent, CommitConflict, ConflictClass, RecordAspectPatchTarget,
    RelationEndpointUpdateMissingState,
};

use super::record_aspect_patch;

pub(super) fn apply(
    intent: &ApplyRelationAspectPatchIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let slot = intent.relation_id.slot_index();
    let (kind_id, endpoints, old_state) = workspace.with_context(|context| {
        ensure_relation_target_is_current(context.state, intent.relation_id)?;
        let partition = context
            .state
            .get_current_partition(intent.relation_id.partition_id)
            .ok_or_else(|| missing(intent, RelationEndpointUpdateMissingState::Partition))?;
        let slot_view = partition
            .relation_arena()
            .get_slot(slot)
            .ok_or_else(|| missing(intent, RelationEndpointUpdateMissingState::Slot))?;
        let kind_id = slot_view
            .kind_id()
            .ok_or_else(|| missing(intent, RelationEndpointUpdateMissingState::KindId))?;
        let endpoints = slot_view
            .extra()
            .endpoints
            .clone()
            .ok_or_else(|| missing(intent, RelationEndpointUpdateMissingState::Endpoints))?;
        Ok::<_, CommitConflict>((
            kind_id,
            endpoints,
            slot_view.extra().authoritative_aspect_state.clone(),
        ))
    })?;
    let target = RecordAspectPatchTarget::Relation {
        relation_id: intent.relation_id,
        kind_id,
    };
    let patch = record_aspect_patch::readmit(
        intent.aspect_patch.clone(),
        PortablePatchReadmissionPurpose::RecordMutation,
        workspace.relation_aspect_plan(kind_id),
        target,
    )?;
    let new_state = record_aspect_patch::apply(old_state.as_ref(), &patch, target)?;
    let version_id = workspace.version_id();
    workspace.with_context(|context| {
        context
            .state
            .mark_relation_slot_touched(intent.relation_id.partition_id, slot);
        let partition = context
            .state
            .get_partition_mut(intent.relation_id.partition_id);
        partition.relation_arena.apply_extra_update(
            slot,
            RelationExtra {
                endpoints: Some(endpoints.clone()),
                authoritative_aspect_state: new_state.clone(),
            },
            version_id,
        );
    });
    Ok(MutationOutcome::relation_updated(
        intent.relation_id,
        kind_id,
        endpoints.source,
        endpoints.target,
        endpoints.source,
        endpoints.target,
        old_state,
        new_state,
        Some(patch),
    ))
}

fn missing(
    intent: &ApplyRelationAspectPatchIntent,
    missing: RelationEndpointUpdateMissingState,
) -> CommitConflict {
    CommitConflict::new(ConflictClass::RelationEndpointUpdateStateInconsistency {
        relation_id: intent.relation_id,
        missing,
    })
}
