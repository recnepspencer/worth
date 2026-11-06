//! Fixed history results retain the original SDK readonly delivery or terminal data.
use crate::application_entry::*;
use worth_query_declaration::facade::{
    application_operation::*, application_program::*, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
};
type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;

pub enum WorthQueryFixedHistoryEditOutcome<S, I, P, R, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
{
    Recorded {
        step: crate::application_entry::WorthQueryHistoryStep,
        delivery: WorthQueryClosedFixedDelivery<S, I, P, R, A>,
    },
    AlreadyRecorded {
        step: crate::application_entry::WorthQueryHistoryStep,
        closed: WorthQueryClosedOperation,
    },
    NotRecorded {
        terminal: WorthQueryClosedFixedTerminal<S, I>,
    },
}
