use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationDiscoveredOutputConnection,
    WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryApplicationMutationOutcome, WorthQueryRequiredOutputPreparationDenial};

mod outputs;
mod recovery;

mod selected;
mod start;
pub use outputs::{
    WorthQueryDiscoveredProgramOutputHandle, WorthQueryDiscoveredProgramOutputProgress,
    WorthQueryDiscoveredProgramOutputSettlement,
};
pub use start::WorthQueryDiscoveredOutputStartFailure;

#[derive(Clone, Copy)]
enum DiscoveredRootStartKind {
    Performed,
    Recovery,
}

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

/// What a mutation with discovered program outputs produced: performed, performed but its
/// required outputs could not start, or not performed.
pub enum WorthQueryApplicationDiscoveredMutationOutcome<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    Performed(WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>),
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

/// A landed mutation whose program outputs are discovered from its result.
/// `start_required_outputs` begins settling them.
pub struct WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
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
    discovery: Discovery<Schema, Root>,
    prepared: WorthQueryPreparedRequiredOutputSource,
    program: std::marker::PhantomData<fn() -> Program>,
    retained_source: std::sync::Arc<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
    >,
}

impl<Schema, Intent, Program, Root>
    WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
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

/// A landed mutation with its discovered program outputs started. `required_output_mut`
/// drives them.
pub struct WorthQueryStartedDiscoveredOutputs<Schema, Intent, Program, Root>
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
    required_output: WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
}

impl<Schema, Intent, Program, Root>
    WorthQueryStartedDiscoveredOutputs<Schema, Intent, Program, Root>
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

    /// Transfers the exact result and move-only continuation to a longer-lived owner.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationCommitReceipt,
        MutationResult<Schema, Intent>,
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
    ) {
        (self.receipt, self.result, self.required_output)
    }

    pub fn required_output_mut(
        &mut self,
    ) -> &mut WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root> {
        &mut self.required_output
    }
}

/// Settles a discovered source's commit into its public outcome.
fn discovered_outcome<Schema, Intent, Program, Root>(
    discovery: Discovery<Schema, Root>,
    outcome: WorthQueryApplicationMutationOutcome<
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
        MutationResult<Schema, Intent>,
    >,
    source: super::performed_source::PerformedSourceCommit,
) -> WorthQueryApplicationDiscoveredMutationOutcome<Schema, Intent, Program, Root>
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
                    program: std::marker::PhantomData,
                    discovery,
                    prepared,
                    retained_source,
                },
            )
        }
    }
}
