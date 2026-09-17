use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramRootConnection,
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
type RootConnection<Schema, Program> = ApplicationProgramRootConnection<Schema, Program>;
type ProgramDemand<Schema, Program> =
    <RootConnection<Schema, Program> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type DemandFamily<Schema, Program> =
    <ProgramDemand<Schema, Program> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type DemandSource<Schema, Program> =
    <DemandFamily<Schema, Program> as WorthQueryProducerOutputFamily<Schema>>::Source;
type ProgramRootEdges<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::OutputGraph as ApplicationOutputGraphShape<
        Schema,
    >>::Dependents;

/// Result of a source publication whose required output custody is retained by
/// its installed program.
pub enum WorthQueryApplicationPerformedMutationOutcome<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    Performed(WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program>),
    /// The source publication committed, while its required output could not
    /// yet be admitted. The owner keeps any prepared delivery custody and the
    /// caller still receives the exact source receipt and result.
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
pub struct WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    demand: ProgramDemand<Schema, Program>,
    prepared: WorthQueryPreparedRequiredOutputSource,
    retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
}

impl<'application, Schema, Intent, Program>
    WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Exact committed source evidence required to re-enter owner-held output custody.
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

/// A performed source whose required output demand has been admitted.
pub struct WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    required_output: crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
    >,
}

impl<'application, Schema, Intent, Program>
    WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
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
    > {
        &mut self.required_output
    }
}

pub struct WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
        WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
{
    performed: WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program>,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<'application, Schema, Intent, Program>
    WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>:
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
    ) -> WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program> {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program>,
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
    pub fn execute_performed<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationPerformedMutationOutcome<'application, Schema, Intent, Program>,
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Source = Intent::Binding>,
        <DemandSource<Schema, Program> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = DemandSource<Schema, Program>>,
        <DemandSource<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <DemandSource<Schema, Program> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        <<DemandSource<Schema, Program> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<
                    Schema,
                    <DemandSource<Schema, Program> as ApplicationQueryBinding<Schema>>::Query,
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
        let demand =
            <RootConnection<Schema, Program> as WorthQueryApplicationRequiredOutputConnection<
                Schema,
            >>::demand_from_source(self.request.intent.input())
            .map_err(WorthQueryPerformedMutationExecutionDenial::Connection)?;
        let query_application = self.request.application;
        let query_principal = self.request.principal;
        let query_scope = self.request.scope;
        let query_branch = self.request.branch;
        let preparation_failure = std::cell::RefCell::new(None);
        let prepared_source = std::cell::RefCell::new(None);
        let outcome = self
            .execute_with_preparation_and_commit(
                super::authorization::prepare,
                |_, program, idempotency| match application
                    .compare_and_commit_required_output_source::<Intent::Binding>(
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
                },
            )
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
        let retained_read = crate::application_entry::WorthQueryApplicationReadObservation::new(
            std::sync::Arc::clone(&retained_source),
        );
        let request = crate::application_entry::WorthQueryApplicationRequest {
            application: query_application,
            principal: query_principal,
            scope: query_scope,
            branch: query_branch,
        };
        let output_source = match request
            .at(&retained_read)
            .query(demand.source_intent())
            .execute()
        {
            Ok(source) => source.into_output_demand_source(),
            Err(denial) => {
                application
                    .runtime()
                    .discard_prepared_required_output_source(prepared);
                return Ok(
                    WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                        receipt,
                        result,
                        denial: WorthQueryRequiredOutputPreparationDenial::SourceQuery(denial),
                    },
                );
            }
        };
        if let Err(denial) = application
            .runtime()
            .bind_prepared_required_output_source(&prepared, &output_source)
        {
            application
                .runtime()
                .discard_prepared_required_output_source(prepared);
            return Ok(
                WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied {
                    receipt,
                    result,
                    denial: WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial),
                },
            );
        }
        Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(
            WorthQueryPerformedApplicationMutation {
                receipt,
                result,
                application,
                demand,
                prepared,
                retained_source,
            },
        ))
    }
}

mod denial;
mod start;
pub use denial::*;
