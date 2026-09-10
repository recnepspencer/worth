//! Product branch lifecycle enriched with primary-provider custody.

use std::num::NonZeroUsize;

use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial,
};

use super::WorthQueryApplicationProductBranchCleanup;
use crate::basis::{
    WorthQueryProductBranch, WorthQueryProductBranchCreationRecovery, WorthQueryProductBranchFork,
    WorthQueryProductBranchRecoveryDenial, WorthQueryProductBranches,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub struct WorthQueryApplicationProductBranches<'runtime, Schema> {
    branches: WorthQueryProductBranches<'runtime>,
    pub(super) application: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn current_world(&self) -> WorthQueryProductBranch {
        self.product_runtime.root_product_branch()
    }

    pub fn branches(&self) -> WorthQueryApplicationProductBranches<'_, Schema> {
        WorthQueryApplicationProductBranches {
            branches: self.product_runtime.product_branches(),
            application: self,
        }
    }
}

impl<'runtime, Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'runtime, Schema> {
    pub fn fork(self, source: WorthQueryProductBranch) -> WorthQueryProductBranchFork<'runtime> {
        self.branches.fork(source)
    }

    pub fn recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        self.branches.recovery_page(after, maximum)
    }

    pub fn readmit_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductBranchCreationRecovery, WorthQueryProductBranchRecoveryDenial>
    {
        self.branches.readmit_recovery(handle)
    }

    /// Discovers every Query-owned cleanup obligation and releases only the
    /// application-retirement occurrences held by the primary provider.
    pub fn pending_cleanup(&self) -> Vec<WorthQueryApplicationProductBranchCleanup> {
        for occurrence in self
            .application
            .product_runtime
            .pending_application_retired_product_occurrences()
        {
            self.application
                .primary_provider
                .release_product_occurrence_retention(
                    occurrence.branch(),
                    occurrence.incarnation(),
                );
        }
        let conditional = self.application.bridge.conditional_operations();
        self.application
            .product_runtime
            .pending_owner_cleanup()
            .into_iter()
            .map(|cleanup| {
                WorthQueryApplicationProductBranchCleanup::new(
                    cleanup,
                    std::sync::Arc::clone(&conditional),
                )
            })
            .collect()
    }
}
