//! Only the actual Native interpretation selects a Home Source inventory.
use super::{ManagedIndexSourcePreparation, ManagedIndexSourceScope};
use crate::runtime::RelationalHomeProgramCatalogReceipt;
use worth_foundational::facade::HomeProgramTransitionStepAssociation;

impl ManagedIndexSourcePreparation<'_> {
    pub fn home_catalog(&self) -> Option<RelationalHomeProgramCatalogReceipt> {
        self.interpretation
            .execution_catalog
            .as_ref()
            .map(|meaning| meaning.catalog_receipt())
    }

    pub fn home_target_step(&self) -> Option<HomeProgramTransitionStepAssociation> {
        self.interpretation
            .home_target
            .as_ref()
            .map(|target| target.step)
    }
}

impl ManagedIndexSourceScope<'_> {
    pub fn home_catalog(&self) -> Option<RelationalHomeProgramCatalogReceipt> {
        self.interpretation
            .execution_catalog
            .as_ref()
            .map(|meaning| meaning.catalog_receipt())
    }

    /// At this positioned target step, `proposed()` supplies the complete real
    /// candidate. Its rows can be acquired through the existing Source owner;
    /// no predecessor cache is claimed to be empty or already compatible.
    pub fn home_target_step(&self) -> Option<HomeProgramTransitionStepAssociation> {
        self.interpretation
            .home_target
            .as_ref()
            .map(|target| target.step)
    }
}
