use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
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

type MutationResult<Schema, Binding> = <Binding as ApplicationMutationBinding<Schema>>::Result;
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
pub enum WorthQueryApplicationPerformedMutationOutcome<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    Performed(WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root>),
    /// Original required-root preparation and native partial remain owned.
    ProductUnpublished(
        WorthQueryUnpublishedRequiredApplicationMutation<Schema, Binding, Program, Root>,
    ),
    /// The source publication committed before required-output custody could
    /// be prepared. The caller still receives the exact receipt and result.
    RequiredOutputDenied(WorthQueryRequiredOutputRetentionFailure<Schema, Binding, Program, Root>),
    /// Performed or unresolved native outcome whose recovery is not supported here.
    Blocked(
        super::performed_source::WorthQueryBlockedProgramSource<
            <Binding as ApplicationMutationBinding<Schema>>::Denial,
            MutationResult<Schema, Binding>,
            ProgramDemand<Schema, Root>,
        >,
    ),
    NotPerformed(
        WorthQueryApplicationMutationOutcome<
            <Binding as ApplicationMutationBinding<Schema>>::Denial,
            MutationResult<Schema, Binding>,
        >,
    ),
}

/// Fresh source result that can start its installed required outputs exactly once.
pub struct WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    result: MutationResult<Schema, Binding>,
    source: preparation::WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>,
}

impl<Schema, Binding, Program, Root>
    WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub(in crate::application_entry) fn into_preparation(
        self,
    ) -> (
        MutationResult<Schema, Binding>,
        preparation::WorthQueryRequiredOutputPreparation<Schema, Program, Root, Binding>,
    ) {
        (self.result, self.source)
    }

    /// Exact committed source evidence required to re-enter owner-held output custody.
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.source.receipt
    }
}

/// A performed source whose required output demand has been admitted.
pub struct WorthQueryStartedRequiredOutputs<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    receipt: WorthQueryApplicationCommitReceipt,
    result: MutationResult<Schema, Binding>,
    required_output:
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
}

impl<Schema, Binding, Program, Root>
    WorthQueryStartedRequiredOutputs<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Binding> {
        &self.result
    }

    /// Transfers the exact committed result and owned fixed-output continuation.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationCommitReceipt,
        MutationResult<Schema, Binding>,
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
pub struct WorthQueryRequiredOutputStartFailure<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    performed: WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root>,
    denial: WorthQueryRequiredOutputPreparationDenial,
}

impl<Schema, Binding, Program, Root>
    WorthQueryRequiredOutputStartFailure<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub const fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }

    pub const fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.performed.source.receipt
    }

    pub const fn result(&self) -> &MutationResult<Schema, Binding> {
        &self.performed.result
    }

    pub fn into_performed(
        self,
    ) -> WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root> {
        self.performed
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPerformedApplicationMutation<Schema, Binding, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    ) {
        (self.performed, self.denial)
    }
}

/// Settles a performed source's commit into its public outcome: not
/// performed, committed without output custody, or performed with custody.
fn performed_outcome<Schema, Binding, Program, Root>(
    demand: ProgramDemand<Schema, Root>,
    outcome: WorthQueryApplicationMutationOutcome<
        <Binding as ApplicationMutationBinding<Schema>>::Denial,
        MutationResult<Schema, Binding>,
    >,
    source: super::performed_source::PerformedSourceCommit,
) -> WorthQueryApplicationPerformedMutationOutcome<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
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
