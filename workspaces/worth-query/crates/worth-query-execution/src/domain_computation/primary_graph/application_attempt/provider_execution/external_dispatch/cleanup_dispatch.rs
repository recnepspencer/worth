//! Resume the genuine original delivery retained across failed native cleanup.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationManagedCleanupReceipt as Receipt,
    WorthQueryCleanupDispatchDenial as Denial, WorthQueryCleanupDispatchPosture as Posture,
};
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Finish only this owner's original first-send obligation. This cannot
    /// accept recovered receipt data or a descriptive cleanup copy. Any physical
    /// attempt, including unwind or lost response, consumes first-send eligibility.
    pub fn resume_cleanup_original_dispatch<'a>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        receipt: &'a mut Receipt,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<Option<&'a WorthQueryExternalEffectDispatch>, Denial> {
        match receipt.posture {
            Posture::NotDeclared => return Ok(None),
            Posture::DiagnosticCopy => return Err(Denial::OriginalNotOwned),
            Posture::AttemptedUnresolved | Posture::Observed => {
                return Err(Denial::AlreadyAttempted)
            }
            Posture::TerminalObserved => return Err(Denial::AlreadyCompleted),
            Posture::Pending => {}
        }
        if request.cancellation().is_cancelled() {
            return Err(Denial::Cancelled);
        }
        if request
            .deadline()
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
        {
            return Err(Denial::TimedOut);
        }
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|denial| Denial::ExecutionDenied(denial.into()))?;
        let original = receipt
            .original
            .as_ref()
            .ok_or(Denial::OriginalNotOwned)?
            .original();
        if original.original_authority().runtime_authority() != self.runtime.authority_identity() {
            return Err(Denial::ForeignRuntime);
        }
        let committed = self
            .primary_provider
            .original_attempt_dispatch_outbox(&execution, original)
            .map_err(Denial::OwnerReadDenied)?
            .ok_or(Denial::OriginalNotOwned)?;
        let transport = self
            .external_effect_transport
            .get()
            .ok_or(Denial::TransportNotInstalled)?;
        let result = self.perform_committed_external_dispatch(
            &execution,
            phase,
            transport.as_ref(),
            committed,
            request,
            |event| match event {
                OriginalDispatchObservation::Starting => {
                    receipt.posture = Posture::AttemptedUnresolved
                }
                OriginalDispatchObservation::Observed(dispatch) => {
                    receipt.posture = Posture::Observed;
                    receipt.dispatch = Some(dispatch.clone());
                }
            },
        );
        if matches!(
            &result,
            Err(WorthQueryExternalDispatchPreparationDenial::AlreadyCompleted)
        ) && receipt.posture == Posture::Pending
        {
            receipt.posture = Posture::TerminalObserved;
        }
        result.map_err(Denial::DispatchDenied)?;
        Ok(receipt.dispatch.as_ref())
    }
}
