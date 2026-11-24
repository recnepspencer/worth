mod borrowed_evidence;
pub(crate) use borrowed_evidence::{BorrowedAspectDeltaEvidence, BorrowedPatchOperation};
mod aspect_trace_projection;
mod changed_authoritative_patch;
pub(crate) mod data;
pub(super) mod engine;
pub(super) mod field_changes;
mod lifecycle_transition_evidence;
mod materialized_state;
mod patch_authority;
mod patch_fragments;
mod publication_fragment;
pub(super) mod published_patch_projection;
pub(super) use publication_fragment::canonical_publication_fragment;
mod semantic_change_projection;

#[cfg(test)]
mod engine_tests;
#[cfg(test)]
mod patch_fragments_tests;

#[cfg(test)]
use data::CanonicalDeltaError;
pub(crate) use data::{
    CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding,
};
#[cfg(test)]
pub(crate) use patch_fragments::authoritative_patch_with_delta_supplements;
pub(crate) use patch_fragments::FoundationalPatchFragment;
