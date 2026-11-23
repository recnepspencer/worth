mod commit_births;
mod commit_finalization;
mod discriminator_scan;
mod event_emission;
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
#[cfg(test)]
pub(crate) use commit_finalization::{finalize_postures_in_working_state, test_finalize_with_schema};
pub(crate) use discriminator_scan::extract_embedded_name_refs;
#[cfg(test)]
pub(crate) use name_minting::PersistentNameMintingAuthority;
pub(crate) use name_readmission::{
    verify_persisted_birth_digest, BirthReadmissionRefusal, PersistedBirthClaim,
    PersistedNameBytes, VerifiedBirthDigest,
};

use crate::runtime::RelationalRuntime;

pub struct LineageAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

pub(crate) struct LineagePreparationAuthority<'runtime> {
    runtime: &'runtime crate::runtime::RelationalPreparationRuntime,
}

impl RelationalRuntime {
    pub(crate) fn lineage_authority(&self) -> LineageAuthority<'_> {
        LineageAuthority::new(self)
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn lineage_preparation_authority(&self) -> LineagePreparationAuthority<'_> {
        LineagePreparationAuthority::new(self)
    }
}

impl<'runtime> LineagePreparationAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime crate::runtime::RelationalPreparationRuntime) -> Self {
        Self { runtime }
    }
}

impl<'runtime> LineageAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
