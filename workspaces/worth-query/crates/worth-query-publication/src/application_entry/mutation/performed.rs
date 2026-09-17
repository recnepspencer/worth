use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramOutputsShape,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
    WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryMutationSourcePrepared,
};

type MutationResult<Schema, Intent> =
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Result;
type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
type ProgramDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type DemandFamily<Schema, Root> =
    <ProgramDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type DemandSource<Schema, Root> =
    <DemandFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;

/// Result of a source publication whose required output custody is retained by
/// its installed program.
pub enum WorthQueryApplicationPerformedMutationOutcome<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    Performed(WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>),
    /// The source publication committed before required-output custody could
    /// be prepared. The caller still receives the exact receipt and result.
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

/// Fresh source result that can start its installed required outputs exactly once.
pub struct WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    demand: ProgramDemand<Schema, Root>,
    prepared: WorthQueryPreparedRequiredOutputSource,
    retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
    source_bound: bool,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Exact committed source evidence required to re-enter owner-held output custody.
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

/// A performed source whose required output demand has been admitted.
pub struct WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    required_output: crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
        Root,
    >,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.result
    }

    pub fn required_output_mut(
        &mut self,
    ) -> &mut crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
        Root,
    > {
        &mut self.required_output
    }
}

pub struct WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    performed: WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<'application, Schema, Intent, Program, Root>
    WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    pub const fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.performed.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.performed.result
    }

    pub fn into_performed(
        self,
    ) -> WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root> {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    ) {
        (self.performed, self.denial)
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
    pub fn execute_performed<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationPerformedMutationOutcome<'application, Schema, Intent, Program, Root>,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
        <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Root>>,
        <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                    Schema,
                    <DemandSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query,
                > + Clone,
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
        let demand =
            <RootConnection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<
                Schema,
            >>::demand_from_source(self.request.intent.input())
            .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
        let preparation_failure = std::cell::RefCell::new(None);
        let prepared_source = std::cell::RefCell::new(None);
        let outcome = self
            .execute_with_commit(true, |_, program, idempotency| {
                match application
                    .compare_and_commit_required_output_source::<Root, Intent::Binding>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        program,
                        idempotency,
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
            return Ok(WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
                outcome,
            ));
        };
        if let Some(failure) = preparation_failure.into_inner() {
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
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
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery,
                },
            );
        };
        Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(
            WorthQueryPerformedApplicationMutation {
                receipt,
                result,
                application,
                demand,
                prepared,
                retained_source,
                source_bound: false,
            },
        ))
    }
}

mod denial;
mod start;
pub use denial::*;
