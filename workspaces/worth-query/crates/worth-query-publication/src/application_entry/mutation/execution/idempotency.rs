//! Recorded idempotency resolution preserves historical outcomes.
use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn resolve_idempotency(
        &self,
        phase: &AdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        Option<
            WorthQueryApplicationMutationOutcome<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
            >,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        self.request
            .application
            .validate_application_advancement(phase)
            .map_err(|cause| {
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause.into())
            })?;
        let resolution = match self
            .request
            .application
            .resolve_admitted_application_idempotency(phase, admission, idempotency)
        {
            Ok(read) => read.into_resolution(),
            Err(denial) => {
                return match denial.historical_commit() {
                    Some(observation) => Ok(Some(
                        WorthQueryApplicationMutationOutcome::PreviouslyCommitted(observation),
                    )),
                    None => Err(WorthQueryApplicationRequestMutationDenial::Idempotency(
                        denial,
                    )),
                };
            }
        };
        Ok(match resolution {
            WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => Some(
                WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
            ),
            WorthQueryApplicationIdempotencyResolution::IntentDrift => {
                Some(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
            }
            WorthQueryApplicationIdempotencyResolution::Unseen => None,
        })
    }
}
