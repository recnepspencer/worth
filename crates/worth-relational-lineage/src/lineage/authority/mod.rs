pub mod commit_births;
pub mod commit_finalization;
pub mod commit_name_preview;
pub mod committed_allocations;
use worth_relational_contracts::internal_port::lineage::data::discriminator_scan;
mod mint_order;
pub mod name_minting;
pub mod name_readmission;
pub mod posture_allocation_drafts;
pub mod posture_continuity;
pub mod posture_drafts;
pub mod posture_events;
pub mod posture_finalization;
pub mod posture_merge_validation;
pub mod posture_split_validation;
pub mod posture_validation;
pub mod selection_continuity;
pub mod staged_effects;

#[cfg(test)]
mod commit_name_preview_continuity_tests;
#[cfg(test)]
mod commit_name_preview_tests;
#[cfg(test)]
mod mint_order_tests;
#[cfg(test)]
pub mod name_minting_tests;
#[cfg(test)]
mod pending_birth_identity;

pub use commit_births::extract_committed_name_births;
pub use commit_finalization::resolve_commit_naming_plan;
pub use commit_name_preview::{
    preview_commit_names, CommitNamePreviewInput, PendingCommitNames, PendingRecordName,
};
pub use posture_finalization::prepare_posture_finalization;
#[doc(hidden)]
pub mod finalization_test_support;
pub use discriminator_scan::extract_embedded_name_refs;
pub use finalization_test_support::{
    finalize_postures_in_working_state_for_test, finalize_with_schema_for_test,
};
pub use name_minting::PersistentNameMintingAuthority;
pub use name_readmission::verify_persisted_birth_digest;
pub(crate) use name_readmission::BirthReadmissionRefusal;
pub use name_readmission::PersistedBirthClaim;
pub(crate) use name_readmission::PersistedNameBytes;
pub(crate) use name_readmission::VerifiedBirthDigest;

#[doc(hidden)]
pub mod posture_fixture;

#[cfg(test)]
mod untouched_keeper_merge;

#[cfg(test)]
mod posture_lineage_tests;
