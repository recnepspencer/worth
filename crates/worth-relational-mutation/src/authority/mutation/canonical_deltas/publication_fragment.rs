//! One canonical delta and its exact patch produce the portable record fragment.
use super::data::CanonicalRecordAspectDelta;
use super::patch_fragments::{
    authoritative_patch_with_delta_supplements, published_patch_for_delta,
    FoundationalPatchFragment,
};
use super::semantic_change_projection::semantic_changes;
use crate::authority::mutation::outcomes::mutation_aspect_patch::MutationAspectPatch;
use worth_relational_contracts::internal_port::publication::patch::data::patch_detail::PatchDetail;

pub(crate) fn canonical_publication_fragment(
    canonical_delta: &CanonicalRecordAspectDelta,
    detail: PatchDetail,
    authoritative_patch: Option<MutationAspectPatch>,
) -> Result<
    FoundationalPatchFragment,
    worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflict,
>{
    Ok(match authoritative_patch {
        Some(patch) => {
            let patch = authoritative_patch_with_delta_supplements(canonical_delta, patch)
                .map_err(|error| error.to_commit_conflict())?;
            let published_patch = published_patch_for_delta(canonical_delta, &patch)
                .map_err(|error| error.to_commit_conflict())?;
            FoundationalPatchFragment {
                target: canonical_delta.target.clone(),
                structural_change: canonical_delta.structural_change,
                semantic_changes: semantic_changes(canonical_delta),
                contains_opaque_aspect: canonical_delta.contains_opaque_aspect,
                detail,
                relation_birth: None,
                patch,
                published_patch,
            }
        }
        None => canonical_delta
            .clone()
            .into_foundational_patch_fragment(detail)
            .map_err(|error| error.to_commit_conflict())?,
    })
}
