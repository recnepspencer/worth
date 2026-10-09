use super::*;

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Runs this evaluation under the caller's active execution request.
    pub fn execute(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        evaluation: &SignalConditionalEvaluationAdmission,
        request: SignalConditionalServiceExecutionRequest,
        condition: &mut impl InstalledSignalConditionResolver,
        comparator: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial> {
        super::super::request_completion::run_conditional_request(execution, |work| {
            self.execute_in_request(work, evaluation, request, condition, comparator, compute)
        })
        .map_err(SignalConditionalServiceExecutionDenial::SlotAdmission)?
    }

    fn execute_in_request(
        &self,
        work: &mut worth_execution::MapKernelContext<'_, '_>,
        evaluation: &SignalConditionalEvaluationAdmission,
        request: SignalConditionalServiceExecutionRequest,
        condition: &mut impl InstalledSignalConditionResolver,
        comparator: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial> {
        use SignalConditionalServiceExecutionDenial as Denial;

        if !Arc::ptr_eq(&self.authority, &evaluation.service_authority) {
            return Err(Denial::DefinitionMismatch);
        }
        let owner = SignalOwner::upgrade(&self.owner).map_err(Denial::OwnerUnavailable)?;
        let admission = owner
            .admit()
            .map_err(map_observation_admission_denial)
            .map_err(Denial::OwnerAdmission)?;
        let branch = self.basis.owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| Denial::OwnerAdmission(map_basis_registry_denial(denial, branch)))?;
        if cell.incarnation() != self.incarnation {
            return Err(Denial::StaleBasisAdmission);
        }
        let mut evaluation_state = match evaluation.execution.try_lock() {
            Ok(execution) => execution,
            Err(TryLockError::WouldBlock) => return Err(Denial::SlotBusy),
            Err(TryLockError::Poisoned(_)) => return Err(Denial::SlotPoisoned),
        };
        let mut kernel_request = SignalConditionalExecutionRequest::new(
            &evaluation.contract,
            evaluation.source.projection(),
            evaluation.execution_identity.as_ref(),
            request.attempt,
        );
        if request.force_on_demand {
            kernel_request = kernel_request.force_on_demand();
        }
        let slot_reused = evaluation_state.slot.is_some();
        let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.execute_conditional(
                work,
                &admission,
                &self.basis,
                &self.definition,
                evaluation,
                &mut evaluation_state,
                kernel_request,
                condition,
                comparator,
                compute,
            )
        }));
        match execution {
            Ok(Ok(completion)) => Ok(completion.with_slot_reuse(slot_reused)),
            Ok(Err(denial)) => Err(denial),
            Err(payload) => {
                drop(evaluation_state);
                std::panic::resume_unwind(payload)
            }
        }
    }
}
