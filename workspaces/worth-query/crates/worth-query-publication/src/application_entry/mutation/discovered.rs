use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryApplicationMutationOutcome, WorthQueryRequiredOutputPreparationDenial};

mod outputs;
mod recovery;
mod resolve;
mod selected;
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

/// Settles a discovered source's commit into its public outcome.
fn discovered_outcome<'application, Schema, Intent, Program, Root>(
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    discovery: Discovery<Schema, Root>,
    outcome: WorthQueryApplicationMutationOutcome<
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
        MutationResult<Schema, Intent>,
    >,
    source: super::performed_source::PerformedSourceCommit,
) -> WorthQueryApplicationDiscoveredMutationOutcome<'application, Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        return WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(outcome);
    };
    match source.into_custody() {
        Err(denial) => WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied {
            receipt,
            result,
            denial,
        },
        Ok((prepared, retained_source)) => {
            WorthQueryApplicationDiscoveredMutationOutcome::Performed(
                WorthQueryPerformedDiscoveredApplicationMutation {
                    receipt,
                    result,
                    application,
                    discovery,
                    prepared,
                    retained_source,
                },
            )
        }
    }
}
