use crate::authority::mutation::outcomes::MutationOutcome;
use crate::authority::mutation::stale_targets::{
    ensure_entity_target_is_current, ensure_relation_target_is_current,
};
use crate::authority::mutation::MutationWorkspace;
use crate::authority::mutation::{apply_adjacency_deltas, AdjacencyDelta, AdjacencyDeltaKind};
use crate::storage::overlay::PartitionAccess;
use crate::storage::substrate::{RelationEndpoints, RelationExtra};
use crate::transactions::data::{
    CommitConflict, ConflictClass, EntityReference, RecordAspectPatchTarget,
    RelationEndpointUpdateMissingState, UpdateRelationEndpointsIntent,
};
use worth_foundational::facade::{PortablePatchReadmissionPurpose, PortableRecordAspectPatch};

use super::{record_aspect_patch, relation_endpoint_candidate};

pub(super) fn apply(
    intent: &UpdateRelationEndpointsIntent,
    workspace: &mut MutationWorkspace<'_>,
) -> Result<MutationOutcome, CommitConflict> {
    let version_id = workspace.version_id();
    let slot = intent.relation_id.slot_index();
    let source = resolve_relation_endpoint(workspace, &intent.source, "source")?;
    let target = resolve_relation_endpoint(workspace, &intent.target, "target")?;
    let (kind_id, old_endpoints, old_authoritative_aspect_state) =
        workspace.with_context(|context| {
            ensure_relation_target_is_current(context.state, intent.relation_id)?;
            if let EntityReference::Existing(existing_source) = &intent.source {
                ensure_entity_target_is_current(context.state, *existing_source)?;
            }
            if let EntityReference::Existing(existing_target) = &intent.target {
                ensure_entity_target_is_current(context.state, *existing_target)?;
            }
            let partition = context
                .state
                .get_partition(intent.relation_id.partition_id)
                .ok_or_else(|| {
                    CommitConflict::new(ConflictClass::RelationEndpointUpdateStateInconsistency {
                        relation_id: intent.relation_id,
                        missing: RelationEndpointUpdateMissingState::Partition,
                    })
                })?;
            let slot_view = partition.relation_arena.get_slot(slot).ok_or_else(|| {
                CommitConflict::new(ConflictClass::RelationEndpointUpdateStateInconsistency {
                    relation_id: intent.relation_id,
                    missing: RelationEndpointUpdateMissingState::Slot,
                })
            })?;
            let kind_id = slot_view.kind_id().ok_or_else(|| {
                CommitConflict::new(ConflictClass::RelationEndpointUpdateStateInconsistency {
                    relation_id: intent.relation_id,
                    missing: RelationEndpointUpdateMissingState::KindId,
                })
            })?;
            if kind_id != intent.kind_id {
                return Err(CommitConflict::new(
                    ConflictClass::RelationEndpointUpdateKindMismatch {
                        relation_id: intent.relation_id,
                        intent_kind_id: intent.kind_id,
                        authoritative_kind_id: kind_id,
                    },
                ));
            }
            let old_endpoints = slot_view.extra().endpoints.clone().ok_or_else(|| {
                CommitConflict::new(ConflictClass::RelationEndpointUpdateStateInconsistency {
                    relation_id: intent.relation_id,
                    missing: RelationEndpointUpdateMissingState::Endpoints,
                })
            })?;
            let old_authoritative_aspect_state =
                slot_view.extra().authoritative_aspect_state.clone();

            context
                .state
                .mark_relation_slot_touched(intent.relation_id.partition_id, slot);
            apply_adjacency_deltas(
                context.state,
                &[
                    AdjacencyDelta {
                        relation_id: intent.relation_id,
                        kind_id,
                        kind: AdjacencyDeltaKind::Deleted {
                            source: old_endpoints.source,
                            target: old_endpoints.target,
                        },
                    },
                    AdjacencyDelta {
                        relation_id: intent.relation_id,
                        kind_id,
                        kind: AdjacencyDeltaKind::Created { source, target },
                    },
                ],
                version_id,
            );
            Ok::<_, CommitConflict>((kind_id, old_endpoints, old_authoritative_aspect_state))
        })?;
    let patch_target = RecordAspectPatchTarget::Relation {
        relation_id: intent.relation_id,
        kind_id,
    };
    let plan = workspace.relation_aspect_plan(kind_id);
    let candidate = relation_endpoint_candidate::append_authoritative_endpoints(
        PortableRecordAspectPatch::new([]),
        plan,
        source,
        target,
    );
    let authoritative_patch = record_aspect_patch::readmit(
        candidate,
        PortablePatchReadmissionPurpose::RecordMutation,
        plan,
        patch_target,
    )?;
    let authoritative_aspect_state = record_aspect_patch::apply(
        old_authoritative_aspect_state.as_ref(),
        &authoritative_patch,
        patch_target,
    )?;
    workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut(intent.relation_id.partition_id);
        partition.relation_arena.apply_extra_update(
            slot,
            RelationExtra {
                endpoints: Some(RelationEndpoints { source, target }),
                authoritative_aspect_state: authoritative_aspect_state.clone(),
            },
            version_id,
        );
    });

    Ok(MutationOutcome::relation_updated(
        intent.relation_id,
        kind_id,
        old_endpoints.source,
        old_endpoints.target,
        source,
        target,
        old_authoritative_aspect_state,
        authoritative_aspect_state,
        record_aspect_patch::published_patch(authoritative_patch),
    ))
}

fn resolve_relation_endpoint(
    workspace: &MutationWorkspace<'_>,
    entity_reference: &EntityReference,
    label: &str,
) -> Result<crate::identity::data::EntityId, CommitConflict> {
    workspace
        .resolve_entity_reference(entity_reference)
        .ok_or_else(|| {
            CommitConflict::new(ConflictClass::InvalidRelationEndpoint {
                detail: format!(
                    "relation endpoint update requires a live or same-batch created {label} entity"
                ),
            })
        })
}
