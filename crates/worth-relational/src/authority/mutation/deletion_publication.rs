//! Reconstruct deletion publication through the same planners as live mutation.
use super::{canonical_deltas::canonical_delta_for_mutation, outcomes::RecordMutation};
use crate::capabilities::AspectPlanSource;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::publication::patch::data::{PatchDetail, PublishedAuthoritativePatch};
use crate::transactions::data::CommitConflict;
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(crate) fn entity_deletion_publication_patch(
    id: EntityId,
    kind_id: KindId,
    state: Option<&AuthoritativeRecordAspectState>,
    declarations: &crate::schema::data::KindAspectContractDeclarations,
    plans: &impl AspectPlanSource,
    detail: PatchDetail,
) -> Result<PublishedAuthoritativePatch, CommitConflict> {
    let authoritative_patch = super::intents::entity_authoritative_deletion_patch::plan_entity_authoritative_deletion_patch(state, declarations)
        .map_err(|denial| CommitConflict::new(crate::transactions::data::ConflictClass::EntityAuthoritativeAspectStateDenied { kind_id, denial }))?;
    let mutation = RecordMutation::EntityDeleted {
        entity_id: id,
        kind_id,
        old_authoritative_aspect_state: None,
        authoritative_patch: authoritative_patch
            .map(super::outcomes::MutationAspectPatch::from_ordinary),
    };
    let delta = canonical_delta_for_mutation(&mutation, plans)
        .map_err(|error| error.to_commit_conflict())?;
    let RecordMutation::EntityDeleted {
        authoritative_patch,
        ..
    } = mutation
    else {
        unreachable!("this function constructed the exact entity deletion variant")
    };
    Ok(super::effect_publication::canonical_publication_fragment(
        &delta,
        detail,
        authoritative_patch,
    )?
    .published_patch)
}

pub(crate) fn relation_deletion_publication_patch(
    id: RelationId,
    kind_id: KindId,
    source: EntityId,
    target: EntityId,
    state: Option<&AuthoritativeRecordAspectState>,
    plans: &impl AspectPlanSource,
    detail: PatchDetail,
) -> Result<PublishedAuthoritativePatch, CommitConflict> {
    let delta = super::canonical_deltas::evaluate_relation_deletion_state(
        plans, id, kind_id, source, target, state,
    )
    .map_err(|error| error.to_commit_conflict())?;
    Ok(
        super::effect_publication::canonical_publication_fragment(&delta, detail, None)?
            .published_patch,
    )
}
