use std::num::NonZeroUsize;

use crate::branch::{
    ProductBranchHistoryTraversal, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial,
};
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};

use super::super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> crate::lifecycle::ports::RuntimeWorldObservationService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn admit_product_branch_currentness<'selected>(
        &self,
        expected: &'selected ProductBranchObservation,
    ) -> Result<
        crate::branch::ProductBranchCurrentnessScope<'selected>,
        RuntimeWorldBranchAdmissionDenial,
    > {
        crate::lifecycle::read_observation::record_read();
        if expected.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let cell = self
            .state
            .branches
            .branch_cell_by_lifecycle(expected.lifecycle_incarnation())
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        Ok(crate::branch::ProductBranchCurrentnessScope::owner_issued(
            cell,
            expected,
            operation,
            self.state
                .operation_control
                .currentness_latch_for_test(expected.lifecycle_incarnation()),
        ))
    }

    fn observe_product_branch(
        &self,
        branch: &ProductBranchIdentity,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        crate::lifecycle::read_observation::record_read();
        if branch.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let cell = self
            .state
            .branches
            .branch_cell(branch)
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        cell.observe(&self.state.history, &self.state.retention)
            .map_err(super::map_observation_denial)
    }

    fn observe_product_branch_occurrence(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        crate::lifecycle::read_observation::record_read();
        if occurrence.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let cell = self
            .state
            .branches
            .branch_cell_by_lifecycle(occurrence)
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        cell.observe(&self.state.history, &self.state.retention)
            .map_err(super::map_observation_denial)
    }

    fn select_product_branch_ancestor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &ProductBranchObservation,
        source: &crate::identity::CompositeCommitIdentity,
        maximum: Option<NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchAncestorSelection,
        crate::branch::ProductBranchAncestorSelectionDenial,
    > {
        super::history::select_ancestor(self, execution, target, source, maximum)
    }

    fn open_exact_product_history(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        target: &ProductBranchObservation,
    ) -> Result<crate::branch::ProductExactHistoryCursor, crate::branch::ProductExactHistoryDenial>
    {
        super::exact_history::open(self, execution, target)
    }

    fn prepare_exact_product_history_page<'request, 'authority>(
        &self,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        cursor: &crate::branch::ProductExactHistoryCursor,
    ) -> Result<
        crate::branch::PreparedProductExactHistoryPage<'request, 'authority>,
        crate::branch::ProductExactHistoryDenial,
    > {
        super::exact_history::prepare(self, execution, cursor)
    }

    fn trace_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        occurrence: ProductBranchIncarnation,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial> {
        super::history::trace(self, execution, occurrence, maximum)
    }

    fn continue_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &ProductBranchHistoryTraversal,
        maximum: Option<NonZeroUsize>,
    ) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial> {
        super::history::continue_trace(self, execution, previous, maximum)
    }

    fn observe_product_branch_history_entry(
        &self,
        history: &ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        crate::lifecycle::read_observation::record_read();
        super::history::observe_entry(self, history, index)
    }
}
