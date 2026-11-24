//! Only the actual Native interpretation selects a Home Source inventory.
use super::{ManagedIndexSourcePreparation, ManagedIndexSourceScope};
use worth_foundational::facade::HomeProgramTransitionStepAssociation;
use worth_relational_contracts::internal_port::schema::home_program_catalog_receipt::RelationalHomeProgramCatalogReceipt;

impl ManagedIndexSourcePreparation<'_> {
    pub fn home_catalog(&self) -> Option<RelationalHomeProgramCatalogReceipt> {
        self.interpretation.catalog_receipt.clone()
    }

    pub fn home_target_step(&self) -> Option<HomeProgramTransitionStepAssociation> {
        self.interpretation.home_target_step
    }
}

impl ManagedIndexSourceScope<'_> {
    pub fn home_catalog(&self) -> Option<RelationalHomeProgramCatalogReceipt> {
        self.interpretation.catalog_receipt.clone()
    }

    /// At this positioned target step, `proposed()` supplies the complete real
    /// candidate. Its rows can be acquired through the existing Source owner;
    /// no predecessor cache is claimed to be empty or already compatible.
    pub fn home_target_step(&self) -> Option<HomeProgramTransitionStepAssociation> {
        self.interpretation.home_target_step
    }
}
