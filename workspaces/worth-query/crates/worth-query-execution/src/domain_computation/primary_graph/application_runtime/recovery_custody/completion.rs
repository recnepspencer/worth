//! An acknowledged original external completion settles custody without a fresh dispatch.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchRecordedObservation as Observation;
use crate::domain_computation::application_aftermath::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryTransitionDenial,
};
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// A recorded completion grants terminal claim settlement only. It grants
    /// neither a new physical contact nor fresh Performed dispatch evidence.
    pub fn settle_recorded_recovery_completion<Operation, Input, Scope>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        handle: WorthQueryRecoveryHandle,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<WorthQueryRecoveryHandleBinding, WorthQueryRecoveryTransitionDenial> {
        let check = (|| {
            let execution = phase
                .execution_request_for(&self.runtime.product_runtime)
                .map_err(|cause| Denial::new(Kind::CompletionExecutionRequest(cause.into())))?;
            self.admit_recovery_custody()?;
            self.admit_recovery_effect_authority(&handle, admission)?;
            if !self.runtime.dispatch_journal.is_durable() {
                return Err(Denial::new(Kind::RecoveryNotAdmitted));
            }
            let binding = handle.binding();
            let outbox = binding
                .committed_dispatch_outbox()
                .ok_or_else(|| Denial::new(Kind::DispatchOutboxMissing))?;
            let observed = self
                .observe_committed_dispatch_outbox_for_correlation(
                    &execution,
                    outbox.record().correlation(),
                )
                .map_err(|cause| Denial::new(Kind::DispatchOwnerReadDenied(cause)))?;
            if observed.record() != outbox.record()
                || observed.record_ref() != outbox.record_ref()
                || observed.commit_reference() != binding.commit_reference()
                || observed.committed_product_publication().composite_commit()
                    != binding.committed_product_publication().composite_commit()
            {
                return Err(Denial::new(Kind::CorrelationMismatch));
            }
            match self
                .runtime
                .dispatch_journal
                .custody(outbox.record().correlation().digest())
                .and_then(|custody| custody.observation())
            {
                Some(Observation::Completed) => {}
                Some(Observation::ProvenNoEffect(cause)) => {
                    return Err(Denial::new(Kind::ProvenNoEffect(cause)))
                }
                Some(Observation::Acknowledged | Observation::Unresolved(_)) | None => {
                    return Err(Denial::new(Kind::UnresolvedExternalPosture))
                }
            }
            if outbox.record().inbound().is_some() {
                let terminal = self
                    .runtime
                    .primary_provider
                    .lookup_completed_inbound(outbox.record().correlation())
                    .map_err(|_| Denial::new(Kind::TerminalIndexUnavailable))?
                    .ok_or_else(|| Denial::new(Kind::CompletionPublicationPending))?;
                let publication = binding.committed_product_publication();
                if !terminal.matches_original_dispatch(
                    outbox.record(),
                    binding.commit_reference(),
                    publication.composite_commit(),
                    publication.product_incarnation(),
                ) {
                    return Err(Denial::new(Kind::CorrelationMismatch));
                }
            }
            Ok(())
        })();
        if let Err(denial) = check {
            return Err(WorthQueryRecoveryTransitionDenial::Retained {
                denial,
                handle: Box::new(handle),
            });
        }
        handle.consume(
            phase,
            worth_foundational::facade::RecoveryClaimDisposition::Completed,
        )
    }
}
