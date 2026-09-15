use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
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
    WorthQueryApplicationProjection, WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::performed_outputs::{WorthQueryProgramConnectionPlan, WorthQueryProgramRootConnection};
use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryMutationSourcePrepared,
};

type MutationResult<Schema, Intent> =
    <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<
        Schema,
    >>::Result;
type ProgramDemand<Schema, Root> = <Root as WorthQueryProgramRootConnection<Schema>>::Demand;
type DemandFamily<Schema, Root> =
    <ProgramDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type DemandSource<Schema, Root> =
    <DemandFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;

pub enum WorthQueryApplicationPerformedMutationOutcome<
    'application,
    Schema,
    Intent,
    Program,
    Inventory,
    Root,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    Performed(
        WorthQueryPerformedApplicationMutation<
            'application,
            Schema,
            Intent,
            Program,
            Inventory,
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

pub struct WorthQueryPerformedApplicationMutation<
    'application,
    Schema,
    Intent,
    Program,
    Inventory,
    Root,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    demand: ProgramDemand<Schema, Root>,
    prepared: WorthQueryPreparedRequiredOutputSource,
    retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
    marker: std::marker::PhantomData<fn() -> (Inventory, Root)>,
}

impl<'application, Schema, Intent, Program, Inventory, Root>
    WorthQueryPerformedApplicationMutation<'application, Schema, Intent, Program, Inventory, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }
}

pub struct WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Inventory, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    required_output: crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
        Inventory,
    >,
    marker: std::marker::PhantomData<fn() -> Root>,
}

impl<'application, Schema, Intent, Program, Inventory, Root>
    WorthQueryStartedRequiredOutputs<'application, Schema, Intent, Program, Inventory, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.result
    }

    /// Borrows the in-progress program output handle for read-only status inspection.
    pub const fn required_output(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
        Inventory,
    > {
        &self.required_output
    }

    pub fn required_output_mut(
        &mut self,
    ) -> &mut crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        'application,
        Schema,
        Program,
        Inventory,
    > {
        &mut self.required_output
    }
}

pub struct WorthQueryRequiredOutputStartFailure<
    'application,
    Schema,
    Intent,
    Program,
    Inventory,
    Root,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
{
    performed: WorthQueryPerformedApplicationMutation<
        'application,
        Schema,
        Intent,
        Program,
        Inventory,
        Root,
    >,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<'application, Schema, Intent, Program, Inventory, Root>
    WorthQueryRequiredOutputStartFailure<'application, Schema, Intent, Program, Inventory, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Root: WorthQueryProgramRootConnection<Schema, Source = Intent::Binding>,
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
    ) -> WorthQueryPerformedApplicationMutation<
        'application,
        Schema,
        Intent,
        Program,
        Inventory,
        Root,
    > {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedApplicationMutation<
            'application,
            Schema,
            Intent,
            Program,
            Inventory,
            Root,
        >,
        WorthQueryRequiredOutputPreparationDenial,
    ) {
        (self.performed, self.denial)
    }
}

mod denial;
mod execution;
mod start;
pub use denial::*;
