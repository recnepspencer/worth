use std::sync::{Arc, TryLockError};

use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionResolver, SignalConditionalExecutionRequest,
};
use crate::data::error::SignalError;
use crate::data::output::NodeEvaluationResult;
use crate::logic::transaction::SignalTransaction;

use super::{
    SignalConditionalEvaluationAdmission, SignalConditionalExecutionPort,
    SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial,
    SignalConditionalServiceExecutionRequest,
};

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[allow(clippy::too_many_arguments)]
    pub fn execute_within_transaction<E, Ctx>(
        &self,
        transaction: &mut SignalTransaction<'_, D, I, E, Ctx, T>,
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
        if !transaction.admits_conditional_operation_scope(self) {
            return Err(Denial::NestedOperationScopeMismatch);
        }
        if !evaluation.contract_binding.matches(&evaluation.contract)
            || !evaluation
                .contract_binding
                .is_current_for(transaction.conditional_execution_graph())
        {
            return Err(Denial::DefinitionMismatch);
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
            super::execute_conditional_against_graph(
                transaction.conditional_execution_graph_mut(),
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
