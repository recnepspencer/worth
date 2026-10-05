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

/// Why closing a product branch from an application was refused or left
/// unfinished.
#[derive(Debug)]
pub enum WorthQueryApplicationProductBranchCloseDenial {
    /// The application's pending publication on the branch could not be
    /// settled first. The close did not begin.
    ApplicationSettlementPending,
    BranchCoordinationCapacityExhausted,
    /// An inbound-capable dispatch still owns completion/recovery custody.
    OutstandingExternalEffect,
    /// The product runtime refused to begin the close. Nothing was closed.
    Product(WorthQueryProductBranchCloseDenial),
    /// The close began and the branch's retention was released, but owner
    /// cleanup did not finish. Retry the cleanup the failure carries; it is
    /// also listed by `pending_cleanup`.
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
            .application_branch_commit_lane_for_occurrence(occurrence)
            .map_err(|_| {
                WorthQueryApplicationProductBranchCloseDenial::BranchCoordinationCapacityExhausted
            })?;
        let _coordination = commit_lane.enter();
        self.application
            .primary_provider
            .settle_before_product_retirement(occurrence)
            .map_err(|_| {
                WorthQueryApplicationProductBranchCloseDenial::ApplicationSettlementPending
            })?;
        if self
            .application
            .primary_provider
            .has_outstanding_dispatch_for_branch(occurrence)
        {
            return Err(WorthQueryApplicationProductBranchCloseDenial::OutstandingExternalEffect);
        }
        let pending = self
            .application
            .product_runtime
            .begin_product_branch_close(self.branch, WorthQueryProductBranchCloseScope::Application)
            .map_err(WorthQueryApplicationProductBranchCloseDenial::Product)?;
        self.application
            .output_demands
            .release_product_occurrence(pending.occurrence().incarnation());
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
