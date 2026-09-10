//! Application-scoped close of one Query product occurrence.

use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryProductEntry;
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCloseDenial, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCloseScope,
};

#[derive(Debug)]
pub enum WorthQueryApplicationProductBranchCloseDenial {
    ApplicationSettlementPending,
    Product(WorthQueryProductBranchCloseDenial),
}

impl<Schema: ApplicationSchema> WorthQueryProductEntry<'_, Schema> {
    pub fn close(
        self,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryApplicationProductBranchCloseDenial>
    {
        let _commit_serialization = self
            .application
            .primary_provider
            .serialize_application_commit();
        self.application
            .primary_provider
            .settle_before_product_retirement()
            .map_err(|_| {
                WorthQueryApplicationProductBranchCloseDenial::ApplicationSettlementPending
            })?;
        let pending = self
            .application
            .product_runtime
            .begin_product_branch_close(self.branch, WorthQueryProductBranchCloseScope::Application)
            .map_err(WorthQueryApplicationProductBranchCloseDenial::Product)?;
        self.application
            .primary_provider
            .release_product_occurrence_retention(
                pending.occurrence().branch(),
                pending.occurrence().incarnation(),
            );
        pending
            .finish()
            .map_err(WorthQueryApplicationProductBranchCloseDenial::Product)
    }
}
