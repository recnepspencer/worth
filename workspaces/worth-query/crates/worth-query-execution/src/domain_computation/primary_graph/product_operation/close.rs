//! Application-scoped close of one Query product occurrence.

use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationProductBranchCleanup, WorthQueryApplicationProductBranchCleanupFailure,
    WorthQueryProductEntry,
};
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductBranchCloseDenial, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCloseScope,
};

#[derive(Debug)]
pub enum WorthQueryApplicationProductBranchCloseDenial {
    ApplicationSettlementPending,
    Product(WorthQueryProductBranchCloseDenial),
    OwnerCleanupPending(WorthQueryApplicationProductBranchCleanupFailure),
}

impl<Schema: ApplicationSchema> WorthQueryProductEntry<'_, Schema> {
    pub fn close(
        self,
    ) -> Result<WorthQueryProductBranchCloseReceipt, WorthQueryApplicationProductBranchCloseDenial>
    {
        let occurrence = self.branch.occurrence();
        let commit_lane = self
            .application
            .primary_provider
            .application_branch_commit_lane_for_occurrence(occurrence);
        let _coordination = commit_lane.enter();
        self.application
            .primary_provider
            .settle_before_product_retirement(occurrence)
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
        let (branch, cleanup) = pending.into_cleanup();
        WorthQueryApplicationProductBranchCleanup::new(
            cleanup,
            self.application.bridge.conditional_operations(),
        )
        .retry()
        .map(|receipt| WorthQueryProductBranchCloseReceipt::from_owner_cleanup(branch, receipt))
        .map_err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending)
    }
}
