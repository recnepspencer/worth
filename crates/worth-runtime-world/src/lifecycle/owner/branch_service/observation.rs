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
    fn observe_memory_original_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: worth_relational::facade::durability::ObservedRelationalMemoryCommit,
    ) -> Result<
        crate::history::WorldObservedMemoryPublication,
        crate::history::WorldReadmittedPublicationDenial,
    > {
        use crate::history::WorldReadmittedPublicationDenial as Denial;
        let _call = self.try_admit_call().ok_or(Denial::OwnerUnavailable)?;
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::OwnerUnavailable)?;
        if self.state.durable.is_some() {
            return Err(Denial::ForeignHome);
        }
        self.state
            .history
            .observe_memory_original(execution, self.owner_identity(), effect)
    }
    fn readmitted_native_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
    ) -> Result<
        crate::history::WorldReadmittedPublication,
        crate::history::WorldReadmittedPublicationDenial,
    > {
        self.read_original_native_publication(execution, effect)
    }
    fn readmitted_publication(
        &self,
        identity: &worth_foundational::facade::DurableCompositeCommitIdentity,
    ) -> Result<
        crate::history::WorldReadmittedPublication,
        crate::history::WorldReadmittedPublicationDenial,
    > {
        use crate::history::WorldReadmittedPublicationDenial as Denial;
        if !self.branch_service_is_available() {
            return Err(Denial::OwnerUnavailable);
        }
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::OwnerUnavailable)?;
        let identity =
            crate::identity::readmission::readmit_commit(self.owner_identity(), *identity)
                .ok_or(Denial::ForeignHome)?;
        self.state.history.readmitted_publication(&identity)
    }
    fn observe_root_product_branch(
        &self,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        crate::lifecycle::read_observation::record_read();
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let _operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let cell = self
            .state
            .branches
            .root_cell()
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        cell.observe(&self.state.history, &self.state.retention)
            .map_err(super::map_observation_denial)
    }
    fn product_branch_writers_are_locked_out_for_test(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<bool, RuntimeWorldBranchAdmissionDenial> {
        if expected.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let cell = self
            .state
            .branches
            .branch_cell_by_lifecycle(expected.lifecycle_incarnation())
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        Ok(cell.writers_are_locked_out_for_test())
    }

    fn readmit_product_branch_reference(
        &self,
        description: &crate::branch::ProductBranchReferenceDescription,
    ) -> Result<ProductBranchObservation, crate::branch::ProductBranchReferenceReadmissionDenial>
    {
        use crate::branch::ProductBranchReferenceReadmissionDenial as Denial;
        if description.format_version != crate::branch::PRODUCT_BRANCH_REFERENCE_DESCRIPTION_VERSION
        {
            return Err(Denial::UnsupportedDescriptionVersion {
                supported: crate::branch::PRODUCT_BRANCH_REFERENCE_DESCRIPTION_VERSION,
                actual: description.format_version,
            });
        }
        if !description.matches_owner(self.owner_identity()) {
            return Err(Denial::Branch(
                RuntimeWorldBranchAdmissionDenial::ForeignOwner,
            ));
        }
        let name = crate::branch::ProductBranchName::try_new(description.branch_name.clone())
            .map_err(Denial::InvalidName)?;
        let branch = ProductBranchIdentity::issued(self.owner_identity(), name);
        let observed = self
            .observe_product_branch(&branch)
            .map_err(Denial::Branch)?;
        if description.lifecycle_ordinal != observed.lifecycle_incarnation().ordinal() {
            return Err(Denial::IncarnationChanged);
        }
        if !description.matches_observation(&observed) {
            return Err(Denial::Branch(
                RuntimeWorldBranchAdmissionDenial::StaleSourceHead,
            ));
        }
        Ok(observed)
    }

    fn admit_product_branch_currentness<'selected>(
        &self,
        expected: &'selected ProductBranchObservation,
    ) -> Result<
        crate::branch::ProductBranchCurrentnessScope<'selected>,
        RuntimeWorldBranchAdmissionDenial,
    > {
        crate::lifecycle::read_observation::record_read();
        self.owner_identity()
            .admit_branch_authority(expected.owner_identity())?;
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
        self.owner_identity()
            .admit_branch_authority(branch.owner_identity())?;
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
        self.owner_identity()
            .admit_branch_authority(occurrence.owner_identity())?;
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
        super::history::select_ancestor(
            self,
            crate::history::AncestorSelectionPath::Exact,
            execution,
            target,
            source,
            maximum,
        )
    }

    fn select_retained_product_branch_ancestor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &ProductBranchObservation,
        source: &crate::identity::CompositeCommitIdentity,
        maximum: Option<NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchAncestorSelection,
        crate::branch::ProductBranchAncestorSelectionDenial,
    > {
        super::history::select_ancestor(
            self,
            crate::history::AncestorSelectionPath::Retained,
            execution,
            target,
            source,
            maximum,
        )
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
