//! Original committed source and immutable demand after registry retention refuses.
use super::*;

/// Move-only custody of a committed source whose required outputs could not
/// be retained. Its prederived demand stays paired with the original native
/// preparation and carrier. Borrowed receipt, result and denial access does
/// not permit replacing that demand before a checked retention retry.
pub struct WorthQueryRequiredOutputRetentionFailure<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding:
        WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    pub(super) receipt: WorthQueryApplicationCommitReceipt,
    pub(super) result: MutationResult<Schema, Binding>,
    pub(super) denial: WorthQueryRequiredOutputPreparationDenial,
    pub(super) custody: Option<
        worth_query_execution::facade::primary_graph::WorthQueryRequiredOutputSourcePreparationFailure,
    >,
    pub(super) demand: ProgramDemand<Schema, Root>,
    pub(super) program: std::marker::PhantomData<fn() -> Program>,
}

impl<Schema, Binding, Program, Root>
    WorthQueryRequiredOutputRetentionFailure<Schema, Binding, Program, Root>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding: WorthQueryApplicationRequiredOutputSource<Schema, RootConnection<Schema, Root>>,
{
    /// The exact original publication, distinct from required-output completion.
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    /// The original handler result; retry never invokes that handler again.
    pub fn result(&self) -> &MutationResult<Schema, Binding> {
        &self.result
    }

    /// Why the original required-output retention was refused.
    pub fn denial(&self) -> &WorthQueryRequiredOutputPreparationDenial {
        &self.denial
    }
}
