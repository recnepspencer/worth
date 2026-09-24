mod aspect_trace_projection;
mod changed_authoritative_patch;
mod data;
mod engine;
mod lifecycle_transition_evidence;
mod materialized_state;
mod patch_authority;
mod patch_fragments;
mod published_patch_projection;
mod semantic_change_projection;

#[cfg(test)]
mod engine_tests;
#[cfg(test)]
mod patch_fragments_tests;

pub(crate) use data::{
    AuthoritativePatchDeltaOperation, CanonicalAspectDeltaEvidence, CanonicalDeltaError,
    CanonicalRecordAspectDelta, EvaluatedAspectBinding,
};
pub(crate) use engine::canonical_delta_for_mutation;
pub(crate) use patch_fragments::{
    authoritative_patch_with_delta_supplements, published_patch_for_delta,
    FoundationalPatchFragment,
};
pub(crate) use semantic_change_projection::semantic_changes;
