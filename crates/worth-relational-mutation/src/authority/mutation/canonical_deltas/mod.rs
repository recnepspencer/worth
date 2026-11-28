pub mod borrowed_evidence;
pub(crate) use borrowed_evidence::{BorrowedAspectDeltaEvidence, BorrowedPatchOperation};
pub mod aspect_trace_projection;
pub mod binding_evidence;
pub mod data;
pub mod engine;
pub mod field_changes;
pub mod lifecycle_transition_evidence;
pub mod patch_authority;
pub mod patch_fragments;
pub mod publication_fragment;
pub mod published_patch_projection;
pub(super) use publication_fragment::canonical_publication_fragment;
pub mod semantic_change_projection;

#[cfg(test)]
pub mod engine_tests;
#[cfg(test)]
pub mod patch_fragments_tests;

#[cfg(test)]
use data::CanonicalDeltaError;
pub use data::{CanonicalAspectDeltaEvidence, CanonicalRecordAspectDelta, EvaluatedAspectBinding};
#[cfg(test)]
pub(crate) use patch_fragments::authoritative_patch_with_delta_supplements;
pub use patch_fragments::FoundationalPatchFragment;
