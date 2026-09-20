//! Product branch lifecycle enriched with primary-provider custody.

use std::num::NonZeroUsize;

use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial,
};

use super::WorthQueryApplicationProductBranchCleanup;
use super::{
    WorthQueryOrderedProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverage,
    WorthQueryProgramAdoptionCoverageDenial,
};
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
    /// Admits an exact bounded set of currently live branch occurrences for
    /// explicit non-atomic program adoption. Later-created branches are not
    /// silently added to the issued coverage.
    pub fn program_adoption_coverage(
        &self,
        branches: &[WorthQueryProductBranch],
        maximum_targets: NonZeroUsize,
    ) -> Result<WorthQueryProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverageDenial> {
        let coverage = WorthQueryProgramAdoptionCoverage::issue(branches, maximum_targets)?;
        let unadmitted = self
            .application
            .product_runtime
            .activations
            .first_unadmitted_live_occurrence(
                coverage.branches().iter().map(|branch| branch.occurrence()),
            )
            .map_err(|_| WorthQueryProgramAdoptionCoverageDenial::RegistryUnavailable)?;
        if let Some(unadmitted) = unadmitted {
            let branch = coverage
                .branches()
                .iter()
                .copied()
                .find(|branch| branch.occurrence() == unadmitted)
                .unwrap_or(coverage.branches()[0]);
            return Err(WorthQueryProgramAdoptionCoverageDenial::ForeignOrRetiredTarget { branch });
        }
        Ok(coverage)
    }

    pub fn order_program_adoption_coverage(
        &self,
        coverage: WorthQueryProgramAdoptionCoverage,
        ordered_targets: &[WorthQueryProductBranch],
    ) -> Result<WorthQueryOrderedProgramAdoptionCoverage, WorthQueryProgramAdoptionCoverageDenial>
    {
        coverage.order(
            self.application
                .current_world()
                .occurrence()
                .owner_identity(),
            ordered_targets,
        )
    }

    pub fn fork(self, source: WorthQueryProductBranch) -> WorthQueryProductBranchFork<'runtime> {
        let commit_lane = self
            .application
            .primary_provider
            .application_branch_commit_lane_for_occurrence(source.occurrence());
        self.branches.fork(source).with_application_lifecycle(
            std::sync::Arc::clone(&self.application.primary_provider.graph.output_lineage),
            commit_lane,
        )
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
                .output_demands
                .release_product_occurrence(occurrence.incarnation());
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
