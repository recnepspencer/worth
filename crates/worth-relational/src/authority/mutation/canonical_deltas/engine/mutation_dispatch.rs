use crate::publication::patch::data::RecordStructuralChange;

use crate::authority::mutation::canonical_deltas::data::{
    CanonicalDeltaError, CanonicalRecordAspectDelta,
};
use crate::authority::mutation::outcomes::RecordMutation;
use crate::authority::mutation::MutationWorkspace;

use super::entity_delta::{
    evaluate_entity_delta, evaluate_entity_lifecycle_delta, evaluate_entity_update_delta,
};
use super::relation_delta::{
    evaluate_relation_delta, evaluate_relation_lifecycle_delta, evaluate_relation_update_delta,
};
use super::state_views::{EntityAuthoritativeState, RelationState};

pub(crate) fn canonical_delta_for_mutation(
    mutation: &RecordMutation,
    workspace: &MutationWorkspace<'_>,
) -> Result<CanonicalRecordAspectDelta, CanonicalDeltaError> {
    match mutation {
        RecordMutation::EntityCreated {
            entity_id,
            kind_id,
            authoritative_patch,
            ..
        } => evaluate_entity_lifecycle_delta(
            workspace,
            *entity_id,
            *kind_id,
            authoritative_patch.as_ref(),
            RecordStructuralChange::Created,
        ),
        RecordMutation::EntityUpdated {
            entity_id,
            kind_id,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch,
            ..
        } => evaluate_entity_update_delta(
            workspace,
            *entity_id,
            *kind_id,
            EntityAuthoritativeState {
                authoritative_state: old_authoritative_aspect_state.as_ref(),
            },
            EntityAuthoritativeState {
                authoritative_state: new_authoritative_aspect_state.as_ref(),
            },
            authoritative_patch.as_ref(),
        ),
        RecordMutation::EntityDeleted {
            entity_id,
            kind_id,
            authoritative_patch,
            ..
        } => evaluate_entity_lifecycle_delta(
            workspace,
            *entity_id,
            *kind_id,
            authoritative_patch.as_ref(),
            RecordStructuralChange::Deleted,
        ),
        RecordMutation::EntityMaterializationSuspended {
            entity_id,
            kind_id,
            old_authoritative_aspect_state,
        } => evaluate_entity_delta(
            workspace,
            *entity_id,
            *kind_id,
            EntityAuthoritativeState {
                authoritative_state: old_authoritative_aspect_state.as_ref(),
            },
            EntityAuthoritativeState {
                authoritative_state: None,
            },
            RecordStructuralChange::MaterializationSuspended,
        ),
        RecordMutation::EntityRematerialized {
            entity_id,
            kind_id,
            authoritative_patch,
            ..
        } => evaluate_entity_lifecycle_delta(
            workspace,
            *entity_id,
            *kind_id,
            authoritative_patch.as_ref(),
            RecordStructuralChange::Rematerialized,
        ),
        RecordMutation::RelationCreated {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
            ..
        } => evaluate_relation_lifecycle_delta(
            workspace,
            *relation_id,
            *kind_id,
            *source,
            *target,
            authoritative_patch.as_ref(),
            RecordStructuralChange::Created,
        ),
        RecordMutation::RelationUpdated {
            relation_id,
            kind_id,
            old_source,
            old_target,
            new_source,
            new_target,
            old_authoritative_aspect_state,
            new_authoritative_aspect_state,
            authoritative_patch,
        } => evaluate_relation_update_delta(
            workspace,
            *relation_id,
            *kind_id,
            RelationState {
                source: Some(*old_source),
                target: Some(*old_target),
                authoritative_state: old_authoritative_aspect_state.as_ref(),
            },
            RelationState {
                source: Some(*new_source),
                target: Some(*new_target),
                authoritative_state: new_authoritative_aspect_state.as_ref(),
            },
            authoritative_patch.as_ref(),
        ),
        RecordMutation::RelationDeleted {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_aspect_state,
            ..
        } => evaluate_relation_delta(
            workspace,
            *relation_id,
            *kind_id,
            RelationState {
                source: Some(*source),
                target: Some(*target),
                authoritative_state: authoritative_aspect_state.as_ref(),
            },
            RelationState {
                source: None,
                target: None,
                authoritative_state: None,
            },
            RecordStructuralChange::Deleted,
        ),
        RecordMutation::RelationRetainedForAudit {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_aspect_state,
            ..
        } => evaluate_relation_delta(
            workspace,
            *relation_id,
            *kind_id,
            RelationState {
                source: Some(*source),
                target: Some(*target),
                authoritative_state: authoritative_aspect_state.as_ref(),
            },
            RelationState {
                source: Some(*source),
                target: Some(*target),
                authoritative_state: authoritative_aspect_state.as_ref(),
            },
            RecordStructuralChange::RetainedForAudit,
        ),
        RecordMutation::RelationMaterializationSuspended {
            relation_id,
            kind_id,
            source,
            target,
            old_authoritative_aspect_state,
        } => evaluate_relation_delta(
            workspace,
            *relation_id,
            *kind_id,
            RelationState {
                source: Some(*source),
                target: Some(*target),
                authoritative_state: old_authoritative_aspect_state.as_ref(),
            },
            RelationState {
                source: None,
                target: None,
                authoritative_state: None,
            },
            RecordStructuralChange::MaterializationSuspended,
        ),
        RecordMutation::RelationRematerialized {
            relation_id,
            kind_id,
            source,
            target,
            authoritative_patch,
            ..
        } => evaluate_relation_lifecycle_delta(
            workspace,
            *relation_id,
            *kind_id,
            *source,
            *target,
            authoritative_patch.as_ref(),
            RecordStructuralChange::Rematerialized,
        ),
    }
}
