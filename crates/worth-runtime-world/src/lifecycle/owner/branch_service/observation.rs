use std::num::NonZeroUsize;

use crate::branch::{
    ProductBranchHistoryTraversal, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial,
};
use crate::identity::{ProductBranchIdentity, ProductBranchIncarnation};

use super::super::RuntimeWorldOwnerRoot;
use crate::lifecycle::ports::RuntimeWorldCurrentnessAdmissionStop as Stop;

impl<D, I, E, Ctx, T> crate::lifecycle::ports::RuntimeWorldObservationService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn admit_product_branch_currentness_with_work<'selected>(
        &self,
        expected: &'selected ProductBranchObservation,
        prepare: &mut dyn FnMut(u64) -> bool,
    ) -> Result<crate::branch::ProductBranchCurrentnessScope<'selected>, Stop> {
        if expected.owner_identity() != self.owner_identity() {
            return Err(Stop::Native(
                RuntimeWorldBranchAdmissionDenial::ForeignOwner,
            ));
        }
        if !self.branch_service_is_available() {
            return Err(Stop::Native(
                RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
            ));
        }
        let operation = self
            .reserve_creation_operation()
            .map_err(|()| Stop::Native(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable))?;
        let cell = self
            .state
            .branches
            .branch_cell_by_lifecycle_admitted(expected.lifecycle_incarnation(), prepare)
            .map_err(|stop| match stop {
                crate::branch::registry::ProductBranchCurrentnessLookupStop::AdmissionRefused => {
                    Stop::Preparation
                }
                crate::branch::registry::ProductBranchCurrentnessLookupStop::AccountingOverflow => {
                    Stop::Native(RuntimeWorldBranchAdmissionDenial::CurrentnessAccountingOverflow)
                }
            })?
            .ok_or(Stop::Native(
                RuntimeWorldBranchAdmissionDenial::RetiredBranch,
            ))?;
        Ok(crate::branch::ProductBranchCurrentnessScope::owner_issued(
            cell,
            expected,
            operation,
            #[cfg(feature = "test-operation-control")]
            self.state
                .operation_control
                .currentness_latch(expected.lifecycle_incarnation()),
        ))
    }

    fn observe_product_branch(
        &self,
        branch: &ProductBranchIdentity,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
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

    fn trace_product_branch_ancestry(
        &self,
        occurrence: ProductBranchIncarnation,
        maximum: NonZeroUsize,
    ) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial> {
        super::history::trace(self, occurrence, maximum)
    }

    fn continue_product_branch_ancestry(
        &self,
        previous: &ProductBranchHistoryTraversal,
        maximum: NonZeroUsize,
    ) -> Result<ProductBranchHistoryTraversal, RuntimeWorldBranchAdmissionDenial> {
        super::history::continue_trace(self, previous, maximum)
    }

    fn observe_product_branch_history_entry(
        &self,
        history: &ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        super::history::observe_entry(self, history, index)
    }
}
