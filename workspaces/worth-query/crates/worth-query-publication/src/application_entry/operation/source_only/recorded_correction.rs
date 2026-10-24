//! Recorded publication enters the existing source owner's delivery and recovery spine.
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationAdmissionBinding, ApplicationMutationInputScope,
    ApplicationOwnedMutationIdentities,
};
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryCorrectionIntent;

impl<S, B, P, A, M> WorthQuerySourceOnlyOperation<S, WorthQueryCapturedMutationInput<S, B>, P, A, M>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + ApplicationMutationInputScope<S, B>,
    B::IdempotencyKey: Clone,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    M: WorthQuerySourceDeliveryMode,
{
    pub(in crate::application_entry) fn retain_recorded_correction(
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
        identities: &ApplicationOwnedMutationIdentities<S, B>,
        correction: WorthQueryCorrectionIntent,
    ) -> Result<Self, WorthQueryOperationConstructionDenial> {
        let input = identities.borrow().encoded_input();
        let description = WorthQueryOperationDescription {
            intent: WorthQueryCapturedMutationInput::new(input.clone()),
            encoded_input: Ok(input),
            key: identities.key().clone(),
            preconditions: Default::default(),
            source: None,
            branch,
        };
        let mut operation = Self::construct(description, runtime).map_err(|(_, cause)| cause)?;
        operation.admission.recorded_correction = Some(correction);
        Ok(operation)
    }
}

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
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    pub(in crate::application_entry) fn accept_recorded_publication(
        mut self,
        outcome: WorthQueryApplicationRetainedMutationOutcome<
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
        >,
    ) -> super::advancement::Advance<S, I, P, A, M> {
        let step = match outcome {
            WorthQueryApplicationRetainedMutationOutcome::Committed {
                receipt,
                result,
                retained,
            } => Ok(self.complete(WorthQuerySourceCompletion::Fresh {
                receipt,
                result,
                retained: Some(retained),
            })),
            WorthQueryApplicationRetainedMutationOutcome::Other(outcome) => {
                self.accept_initial(outcome)
            }
        };
        match step {
            Ok(SourceStep::Progress) => WorthQuerySourceOnlyAdvance::Progress(self),
            Ok(SourceStep::Completed) => {
                WorthQuerySourceOnlyAdvance::Completed(WorthQueryCompletedSourceOperation {
                    operation: self,
                })
            }
            Ok(SourceStep::Terminal) => {
                WorthQuerySourceOnlyAdvance::Terminal(WorthQueryTerminalSourceOperation {
                    operation: self,
                })
            }
            Err(denial) => WorthQuerySourceOnlyAdvance::Refused {
                operation: self,
                denial,
            },
        }
    }
}
