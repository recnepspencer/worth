mod commit_finalization;
mod discriminator_scan;
mod event_emission;
mod name_minting;
mod posture_drafts;
mod posture_finalization;
mod posture_merge_validation;
mod posture_validation;
mod staged_effects;

#[cfg(test)]
mod name_minting_tests;

pub use name_minting::{PersistentNameMintingAuthority, PERSISTENT_NAME_DOMAIN_TAG};
#[cfg(test)]
pub(crate) use posture_finalization::finalize_postures_in_working_state;

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
