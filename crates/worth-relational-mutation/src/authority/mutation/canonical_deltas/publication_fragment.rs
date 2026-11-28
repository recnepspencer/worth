//! One canonical delta and its exact patch produce the portable record fragment.
use super::data::CanonicalRecordAspectDelta;
use super::patch_fragments::{
    authoritative_patch_with_delta_supplements, FoundationalPatchFragment,
};
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
    match authoritative_patch {
        Some(patch) => {
            authoritative_patch_with_delta_supplements(canonical_delta, patch).and_then(|patch| {
                FoundationalPatchFragment::from_delta(canonical_delta, detail, patch)
            })
        }
        None => canonical_delta.into_foundational_patch_fragment(detail),
    }
    .map_err(|error| error.to_commit_conflict())
}
