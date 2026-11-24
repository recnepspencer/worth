mod commit_births;
mod commit_finalization;
pub(crate) mod committed_allocations;
use crate::lineage::data::discriminator_scan;
mod mint_order;
mod name_minting;
mod name_readmission;
mod posture_allocation_drafts;
mod posture_continuity;
mod posture_drafts;
mod posture_events;
mod posture_finalization;
mod posture_merge_validation;
mod posture_split_validation;
mod posture_validation;
mod selection_continuity;
mod staged_effects;

#[cfg(test)]
mod mint_order_tests;
#[cfg(test)]
mod name_minting_tests;
#[cfg(test)]
mod pending_birth_identity;

pub(crate) use commit_births::extract_committed_name_births;
pub(crate) use commit_finalization::resolve_commit_naming_plan;
pub(crate) use posture_finalization::prepare_posture_finalization;
#[cfg(test)]
mod finalization_test_support;
pub(crate) use discriminator_scan::extract_embedded_name_refs;
#[cfg(test)]
pub(crate) use finalization_test_support::{
    finalize_postures_in_working_state, test_finalize_with_schema,
};
#[cfg(test)]
pub(crate) use name_minting::PersistentNameMintingAuthority;
pub(crate) use name_readmission::{
    verify_persisted_birth_digest, BirthReadmissionRefusal, PersistedBirthClaim,
    PersistedNameBytes, VerifiedBirthDigest,
};
