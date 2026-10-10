//! Retry registry retention for an already committed fixed source, never its handler.
use super::super::*;
use crate::application_entry::mutation::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryApplicationRecoveryRequestDenial,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_program::ApplicationProgramOutputsShape;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolution, WorthQueryManagedApplicationRecoveryDenial,
};
impl<'application, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Only a genuine committed retention refusal can re-enter this path.
    /// Every failed admission or transfer returns the complete owned failure.
    pub fn retry_required_output_retention<Program, Root>(
        self,
        failure: WorthQueryRequiredOutputRetentionFailure<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>,
        (
            WorthQueryApplicationRecoveryRequestDenial,
            WorthQueryRequiredOutputRetentionFailure<Schema, Intent, Program, Root>,
        ),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
    {
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        let mut retained = Some(failure);
        match runtime.with_application_advancement(&scope, |phase| {
            self.retry_required_output_retention_in_advancement(
                &phase,
                retained
                    .take()
                    .expect("opened request owns recovery custody"),
                application,
            )
        }) {
            Ok(outcome) => outcome,
            Err(cause) => Err((
                WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
                retained.take().expect("refused request retains custody"),
            )),
        }
    }

    fn retry_required_output_retention_in_advancement<Program, Root>(
        mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        mut failure: WorthQueryRequiredOutputRetentionFailure<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>,
        (
            WorthQueryApplicationRecoveryRequestDenial,
            WorthQueryRequiredOutputRetentionFailure<Schema, Intent, Program, Root>,
        ),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Intent::Binding:
            WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
    {
        let admitted = (|| {
            let custody = failure.custody.as_ref().ok_or_else(mismatch)?;
            let prepared = self.authorize_required_source::<Program, Root>(
                phase,
                application,
                custody.source(),
            )?;
            let read = self.read_recovery_idempotency(phase, &prepared)?;
            self.check_recovery_liveness()?;
            match read.into_resolution() {
                WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) => {
                    Ok(receipt)
                }
                _ => Err(mismatch()),
            }
        })();
        let published = match admitted {
            Ok(receipt) => receipt,
            Err(denial) => return Err((denial, failure)),
        };
        let custody = failure
            .custody
            .take()
            .expect("admission checked the owned custody");
        let WorthQueryRequiredOutputRetentionFailure {
            receipt,
            result,
            denial,
            demand,
            program,
            ..
        } = failure;
        let (_, _, source, carrier) = custody.into_parts();
        match application.promote_program_output_source(
            &worth_query_execution::publication_boundary::program_publication_access(),
            source,
            carrier,
            published,
        ) {
            Ok((prepared, retained_source)) => Ok(WorthQueryPerformedApplicationMutation {
                result,
                source: super::super::preparation::WorthQueryRequiredOutputPreparation {
                    receipt,
                    demand,
                    prepared,
                    retained_source,
                    source_bound: false,
                    program: std::marker::PhantomData,
                },
            }),
            Err((next_denial, source, carrier)) => {
                let custody = worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure::returned(receipt.clone(), next_denial.clone(), source, carrier);
                Err((
                    WorthQueryApplicationRecoveryRequestDenial::Recovery(
                        WorthQueryManagedApplicationRecoveryDenial::Demand(next_denial),
                    ),
                    WorthQueryRequiredOutputRetentionFailure {
                        receipt,
                        result,
                        denial,
                        custody: Some(custody),
                        demand,
                        program,
                    },
                ))
            }
        }
    }
}
fn mismatch() -> WorthQueryApplicationRecoveryRequestDenial {
    WorthQueryApplicationRecoveryRequestDenial::Recovery(
        WorthQueryManagedApplicationRecoveryDenial::BindingMismatch,
    )
}
