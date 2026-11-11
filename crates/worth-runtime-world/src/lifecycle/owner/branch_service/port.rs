//! Weak service dispatch and same-original creation resumption.
use super::*;
impl<D, I, E, Ctx, T> crate::lifecycle::ports::RuntimeWorldBranchService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn with_home_program_binding(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &ProductBranchObservation,
        required: &worth_proof::HomeProgramTransitionRequired,
        successor: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            &worth_foundational::facade::ApplicationHomeBinding,
        >,
        handoff: &mut dyn for<'owner> FnMut(
            crate::lifecycle::PreparedWorldHomeProgramBinding<'owner>,
        ),
    ) -> Result<(), crate::lifecycle::WorldHomeProgramBindingDenial> {
        self.bind_current_home_program(request, expected, required, successor, handoff)
    }

    fn reserve_creation_name_ordinal(&self) -> Result<u64, RuntimeWorldBranchAdmissionDenial> {
        self.issue_creation_name_ordinal()
    }

    fn create_product_branch(
        &self,
        request: RuntimeWorldBranchCreationRequest<'_, '_, '_>,
    ) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
        match request {
            crate::lifecycle::ports::RuntimeWorldBranchCreationRequest::Memory {
                execution,
                source,
                intent,
                cancellation,
                retained,
            } => {
                self.create_memory_product_branch(execution, source, intent, cancellation, retained)
            }
            crate::lifecycle::ports::RuntimeWorldBranchCreationRequest::Home {
                execution,
                request,
                cancellation,
            } => self.create_durable_product_branch(execution, request, cancellation),
        }
    }

    fn resume_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, crate::recovery::RuntimeWorldRecoveryDenial>
    {
        use crate::recovery::RuntimeWorldRecoveryDenial as Denial;
        self.state.recovery.admit_handle(handle)?;
        let operation = self
            .reserve_creation_operation()
            .map_err(|()| Denial::Busy)?;
        let capacity = self
            .state
            .publication_capacity
            .reserve()
            .map_err(|_| Denial::OutputCapacityExhausted)?;
        let update = self.state.recovery.take_record_for_update(handle)?;
        let mut custody = update.reactivate_creation(operation, capacity)?;
        custody.begin_owner_execution();
        self.drive_durable_creation(execution, custody, cancellation)
            .map_err(|_| Denial::ReopenRequired)
    }

    fn admit_retained_branch_source(
        &self,
        source: &ProductBranchObservation,
    ) -> Result<crate::branch::RetainedProductBranchCreationSource, RuntimeWorldBranchAdmissionDenial>
    {
        self.admit_retained_creation_source(source)
    }

    fn retire_product_branch(
        &self,
        observed: &ProductBranchObservation,
    ) -> Result<ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial> {
        self.retire_observed_branch(observed)
    }
}
