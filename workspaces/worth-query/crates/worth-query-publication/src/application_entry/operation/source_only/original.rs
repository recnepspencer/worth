//! Cold originals resolve native custody; absence never authorizes a replacement effect.
use super::advancement::Step;
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationAdmissionBinding;
use worth_query_execution::publication_boundary::original_operation::WorthQueryOriginalOperationDenial;
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    M: WorthQuerySourceDeliveryMode,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    MutationInput<S, I>: Clone + Send + Sync,
    MutationScope<S, I>: ApplicationMutationScopeResolution<S, MutationPrincipal<S, I>>,
{
    pub(super) fn readmit_original(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<Step, WorthQuerySourceOnlyDenial> {
        use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution as R;
        let read = self
            .admission
            .resolve_keyed_in_advancement(phase, request, None)
            .map_err(WorthQuerySourceOnlyDenial::Recovery)?;
        match read.into_resolution() {
            R::RecoveredCommitted(receipt) => {
                self.admission
                    .description
                    .original
                    .as_mut()
                    .expect("readmitted original")
                    .readmission = false;
                let completion = WorthQuerySourceCompletion::Recorded {
                    retained: None,
                    receipt,
                    publication: None,
                    initial_posture: None,
                    initial_advancement_failure: None,
                };
                if M::RETAIN_COMMIT {
                    self.state = SourceState::<S, I>::RetainedPending(completion);
                    Ok(Step::Progress)
                } else {
                    Ok(self.complete(completion))
                }
            }
            R::ProductUnpublished(partial) => {
                self.state = SourceState::<S, I>::Unpublished(partial.into_recovery());
                self.admission
                    .description
                    .original
                    .as_mut()
                    .expect("readmitted original")
                    .readmission = false;
                Ok(Step::Progress)
            }
            R::Unseen => Err(WorthQuerySourceOnlyDenial::Original(
                WorthQueryOriginalOperationDenial::OwnershipMaterialUnavailable,
            )),
            resolution @ (R::ReleasedOriginal(_)
            | R::IntentDrift
            | R::ProductPreparationPending(_)) => {
                self.keyed_terminal = Some(resolution);
                Ok(Step::Terminal)
            }
        }
    }
}
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Relinquish process aliases while keeping the same durable original for fresh readmission.
    /// Unreleased partial publication must first progress through its native owner.
    pub fn handoff_for_reopen(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            super::super::WorthQueryOriginalOperation,
            WorthQueryOperationCleanupReceipt,
        ),
        (Self, WorthQueryOperationCloseDenial),
    > {
        let original = match self.admission.description.original_lookup() {
            Ok(original) => original,
            Err(cause) => return Err((self, WorthQueryOperationCloseDenial::Original(cause))),
        };
        if let Err(cause) = original.admit(request) {
            return Err((self, WorthQueryOperationCloseDenial::Original(cause)));
        }
        if !matches!(
            self.state,
            SourceState::<S, I>::Initial
                | SourceState::<S, I>::RetryableNoEffect(_)
                | SourceState::<S, I>::RetainedPending(_)
        ) {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        match self.release_cleanup(request) {
            Ok(cleanup) => Ok((original, cleanup)),
            Err(cause) => Err((self, cause)),
        }
    }
}
impl<S, I, P, A, M> WorthQueryCompletedSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn handoff_for_reopen(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            super::super::WorthQueryOriginalOperation,
            WorthQueryOperationCleanupReceipt,
        ),
        (Self, WorthQueryOperationCloseDenial),
    > {
        self.operation
            .handoff_for_reopen(request)
            .map_err(|(operation, cause)| (Self { operation }, cause))
    }
}
