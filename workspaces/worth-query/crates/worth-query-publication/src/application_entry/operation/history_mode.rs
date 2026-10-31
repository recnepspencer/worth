//! Installed action delivery selects the original history operation owner.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationActionDelivery;
mod driver;
mod fixed;
mod source;
pub(in crate::application_entry) use driver::{
    Driver as HistoryOperationDriver, HistoryOperationAdvance, HistoryRecordingSource,
};

pub(in crate::application_entry) type HistoryMode<S, I, A> =
    <Delivery<S, I, A> as ApplicationActionDelivery<S, Binding<S, I>>>::Mode;

/// Sealed availability of a history delivery profile. These are original SDK
/// owners and outcomes, not supplier callbacks or execution permission.
pub trait WorthQueryHistoryOperationMode<S, I, P, A>:
    driver::Driver<
    S,
    I,
    P,
    A,
    Self::Operation,
    Self::Completed,
    Self::Terminal,
    Self::ClosedTerminal,
    Self::Outcome,
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    type Operation;
    type Completed;
    type Terminal;
    type ClosedTerminal;
    type Outcome;
}
