use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryMutationSourcePrepared, WorthQueryPerformedMutationExecutionDenial,
    WorthQueryRequiredOutputPreparationDenial,
};

mod outputs;
mod recovery;
mod resolve;
mod start;
pub use outputs::{
    WorthQueryDiscoveredProgramOutputHandle, WorthQueryDiscoveredProgramOutputProgress,
    WorthQueryDiscoveredProgramOutputSettlement,
};
pub use start::WorthQueryDiscoveredOutputStartFailure;

type MutationResult<Schema, Intent> =
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Result;
type RootConnection<Schema, Root> =
    <<Root as ApplicationOutputGraphShape<Schema>>::RootConnection as ApplicationConnectionShape<
        Schema,
    >>::Binding;
type Discovery<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Discovery;

pub enum WorthQueryApplicationDiscoveredMutationOutcome<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    Performed(
        WorthQueryPerformedDiscoveredApplicationMutation<
            'application,
            Schema,
            Intent,
            Program,
            Root,
        >,
    ),
    RequiredOutputDenied {
        receipt: WorthQueryApplicationCommitReceipt,
        result: MutationResult<Schema, Intent>,
        denial: WorthQueryRequiredOutputPreparationDenial,
    },
    NotPerformed(
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            MutationResult<Schema, Intent>,
        >,
    ),
}

pub struct WorthQueryPerformedDiscoveredApplicationMutation<
    'application,
    Schema,
    Intent,
    Program,
    Root,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    discovery: Discovery<Schema, Root>,
    prepared: WorthQueryPreparedRequiredOutputSource,
    retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryPerformedDiscoveredApplicationMutation<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

pub struct WorthQueryStartedDiscoveredOutputs<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    required_output: WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryStartedDiscoveredOutputs<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.result
    }

    pub fn required_output_mut(
        &mut self,
    ) -> &mut WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root> {
        &mut self.required_output
    }
}

impl<'application, 'principal, 'scope, 'key, Schema, Intent>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        WorthQueryMutationSourcePrepared,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn execute_performed_discovered<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationDiscoveredMutationOutcome<'application, Schema, Intent, Program, Root>,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    {
        if !std::ptr::eq(application.runtime(), self.request.application)
            || application.installed_program().schema_binding()
                != &self
                    .request
                    .application
                    .installed_schema()
                    .binding_identity()
        {
            return Err(WorthQueryPerformedMutationExecutionDenial::ForeignProgram);
        }
        if !application.contains_output_root::<Root>() {
            return Err(WorthQueryPerformedMutationExecutionDenial::UndeclaredOutputRoot);
        }
        let discovery =
            RootConnection::<Schema, Root>::discovery_from_source(self.request.intent.input())
                .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
        let retained_discovery = discovery.clone();
        let preparation_failure = std::cell::RefCell::new(None);
        let prepared_source = std::cell::RefCell::new(None);
        let outcome = self
            .execute_with_commit(true, |_, program, idempotency| {
                match application
                    .compare_and_commit_discovered_output_source::<Root, Intent::Binding>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        program,
                        idempotency,
                        retained_discovery,
                    ) {
                    Ok((outcome, prepared)) => {
                        if let Some(prepared) = prepared {
                            prepared_source.replace(Some(prepared));
                        }
                        outcome
                    }
                    Err(failure) => {
                        let receipt = failure.receipt().clone();
                        preparation_failure.replace(Some(failure));
                        WorthQueryApplicationCommitOutcome::Committed(receipt)
                    }
                }
            })
            .map_err(WorthQueryPerformedMutationExecutionDenial::Mutation)?;
        let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
            return Ok(WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(outcome));
        };
        if let Some(failure) = preparation_failure.into_inner() {
            return Ok(
                WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                        failure.denial().clone(),
                    ),
                },
            );
        }
        let Some((prepared, retained_source)) = prepared_source.into_inner() else {
            return Ok(
                WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery,
                },
            );
        };
        Ok(WorthQueryApplicationDiscoveredMutationOutcome::Performed(
            WorthQueryPerformedDiscoveredApplicationMutation {
                receipt,
                result,
                application,
                discovery,
                prepared,
                retained_source,
            },
        ))
    }
}
