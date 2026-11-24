//! Reconstruct deletion publication through the same planners as live mutation.
use super::canonical_deltas::canonical_publication_fragment;
use super::canonical_deltas::engine::{
    entity_delta::evaluate_entity_lifecycle_delta, relation_delta::evaluate_relation_deletion_state,
};
use super::outcomes::mutation_aspect_patch::MutationAspectPatch;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::publication::patch::data::patch_detail::PatchDetail;
use crate::contracts::publication::patch::data::published_authoritative_patch::PublishedAuthoritativePatch;
use crate::contracts::publication::patch::data::record_structural_change::RecordStructuralChange;
use crate::contracts::schema::aspect_plan_source::AspectPlanSource;
use crate::contracts::transactions::data::outcomes::commit_conflict::{
    CommitConflict, CommitConflictOwnerPort as _,
};
use worth_foundational::facade::AuthoritativeRecordAspectState;

pub(crate) fn entity_deletion_publication_patch(
    id: EntityId,
    kind_id: KindId,
    state: Option<&AuthoritativeRecordAspectState>,
    declarations: &crate::contracts::schema::data::aspect_contracts::kind_declarations::KindAspectContractDeclarations,
    plans: &impl AspectPlanSource,
    detail: PatchDetail,
) -> Result<PublishedAuthoritativePatch, CommitConflict> {
    let authoritative_patch = super::intents::entity_authoritative_deletion_patch::plan_entity_authoritative_deletion_patch(state, declarations)
        .map_err(|denial| CommitConflict::new(crate::contracts::transactions::data::outcomes::conflict_class::ConflictClass::EntityAuthoritativeAspectStateDenied { kind_id, denial }))?;
    let delta = evaluate_entity_lifecycle_delta(
        plans,
        id,
        kind_id,
        authoritative_patch.as_ref(),
        RecordStructuralChange::Deleted,
    )
    .map_err(|error| error.to_commit_conflict())?;
    Ok(canonical_publication_fragment(
        &delta,
        detail,
        authoritative_patch.map(MutationAspectPatch::from_ordinary),
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
    let delta = evaluate_relation_deletion_state(plans, id, kind_id, source, target, state)
        .map_err(|error| error.to_commit_conflict())?;
    Ok(canonical_publication_fragment(&delta, detail, None)?.published_patch)
}

pub mod codec_expectation;
