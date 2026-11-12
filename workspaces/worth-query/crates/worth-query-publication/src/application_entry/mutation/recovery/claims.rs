//! Selected original-intent admission lends its actual funded phase to recovery.
use super::super::authorization::AdmittedMutation;
use super::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding;

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
    /// Authorize the exact original intent and key without executing its handler.
    /// The callback receives current admission and this owner's funded phase;
    /// opaque wire identities cannot replace either input.
    pub fn with_recovery_in_program<Program, R>(
        mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        body: impl for<'step> FnOnce(
            &WorthQueryAdvancementPhase<'step>,
            &AdmittedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationIdempotencyBinding,
            &worth_query_declaration::facade::application_operation::ApplicationEncodedInput<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::InputBinding,
            >,
        ) -> R,
    ) -> Result<R, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        runtime
            .with_application_advancement(&scope, |phase| {
                let (prepared, input) = self.authorize_recovery(&phase, application)?;
                self.check_recovery_liveness()?;
                Ok(body(
                    &phase,
                    &prepared.admission,
                    prepared.idempotency,
                    &input,
                ))
            })
            .map_err(WorthQueryApplicationRecoveryRequestDenial::advancement)?
    }
}
