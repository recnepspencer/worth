use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
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
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryPreparedRequiredOutputSource,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryApplicationMutationOutcome;

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
pub enum WorthQueryApplicationPerformedMutationOutcome<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    Performed(WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>),
    /// Original required-root preparation and native partial remain owned.
    ProductUnpublished(
        WorthQueryUnpublishedRequiredApplicationMutation<Schema, Intent, Program, Root>,
    ),
    /// The source publication committed before required-output custody could
    /// be prepared. The caller still receives the exact receipt and result.
    RequiredOutputDenied(WorthQueryRequiredOutputRetentionFailure<Schema, Intent, Program, Root>),
    /// Performed or unresolved native outcome whose recovery is not supported here.
    Blocked(
        super::performed_source::WorthQueryBlockedProgramSource<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            MutationResult<Schema, Intent>,
            ProgramDemand<Schema, Root>,
        >,
    ),
    NotPerformed(
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            MutationResult<Schema, Intent>,
        >,
    ),
}

/// Fresh source result that can start its installed required outputs exactly once.
pub struct WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    result: MutationResult<Schema, Intent>,
    source:
        preparation::WorthQueryRequiredOutputPreparation<Schema, Program, Root, Intent::Binding>,
}

impl<Schema, Intent, Program, Root>
    WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    /// Exact committed source evidence required to re-enter owner-held output custody.
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.source.receipt
    }
}

/// A performed source whose required output demand has been admitted.
pub struct WorthQueryStartedRequiredOutputs<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Intent>,
    required_output:
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
}

impl<Schema, Intent, Program, Root> WorthQueryStartedRequiredOutputs<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.result
    }

    /// Transfers the exact committed result and owned fixed-output continuation.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationCommitReceipt,
        MutationResult<Schema, Intent>,
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
    ) {
        (self.receipt, self.result, self.required_output)
    }

    pub fn required_output_mut(
        &mut self,
    ) -> &mut crate::application_entry::WorthQueryApplicationProgramOutputHandle<
        Schema,
        Program,
        Root,
    > {
        &mut self.required_output
    }
}

/// A landed mutation whose required outputs could not start. `into_performed` returns the
/// landed mutation; `into_parts` also returns the denial.
pub struct WorthQueryRequiredOutputStartFailure<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    performed: WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<Schema, Intent, Program, Root>
    WorthQueryRequiredOutputStartFailure<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub const fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.performed.source.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Intent> {
        &self.performed.result
    }

    pub fn into_performed(
        self,
    ) -> WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root> {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedApplicationMutation<Schema, Intent, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    ) {
        (self.performed, self.denial)
    }
}

/// Settles a performed source's commit into its public outcome: not
/// performed, committed without output custody, or performed with custody.
fn performed_outcome<Schema, Intent, Program, Root>(
    demand: ProgramDemand<Schema, Root>,
    outcome: WorthQueryApplicationMutationOutcome<
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
        MutationResult<Schema, Intent>,
    >,
    source: super::performed_source::PerformedSourceCommit,
) -> WorthQueryApplicationPerformedMutationOutcome<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Intent::Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    let outcome = match outcome {
        WorthQueryApplicationMutationOutcome::Commit(worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(partial)) => {
            if let Some(pending) = source.take_unpublished() {
                return WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(
                    WorthQueryUnpublishedRequiredApplicationMutation::new(pending, demand, partial));
            }
            WorthQueryApplicationMutationOutcome::Commit(worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::ProductUnpublished(partial))
        }
        other => other,
    };
    if matches!(&outcome, WorthQueryApplicationMutationOutcome::Commit(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::SettlementDeferred(_)
        | worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted::Indeterminate(_))) {
        if let Some(preparation) = source.take_unpublished() {
            return WorthQueryApplicationPerformedMutationOutcome::Blocked(super::performed_source::WorthQueryBlockedProgramSource { _preparation: preparation, payload: demand, outcome });
        }
    }
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        return WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome);
    };
    match source.into_custody() {
        Err((denial, custody)) => {
            WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(
                WorthQueryRequiredOutputRetentionFailure {
                    receipt,
                    result,
                    denial,
                    custody,
                    demand,
                    program: std::marker::PhantomData,
                },
            )
        }
        Ok((prepared, retained_source)) => {
            WorthQueryApplicationPerformedMutationOutcome::Performed(
                WorthQueryPerformedApplicationMutation {
                    result,
                    source: preparation::WorthQueryRequiredOutputPreparation {
                        receipt,
                        program: std::marker::PhantomData,
                        demand,
                        prepared,
                        retained_source,
                        source_bound: false,
                    },
                },
            )
        }
    }
}

mod denial;
mod denied;
mod preparation;
mod recovered;
mod recovery;
mod selected;
mod start;
mod unpublished;
pub use denial::*;
pub use denied::WorthQueryRequiredOutputRetentionFailure;
pub use preparation::WorthQueryRequiredOutputPreparation;
pub use recovered::WorthQueryRecoveredRequiredOutputs;
pub use unpublished::WorthQueryUnpublishedRequiredApplicationMutation;
