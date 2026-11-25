use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
};
use worth_query_installation::facade::{
    WorthQueryTemporalIntentIdempotencyRelation, WorthQueryTemporalOperationInputIdentity,
};

pub use worth_query_execution_lineage::facade::primary_graph::conditional_operation::operation_invocation::{
    WorthQueryTemporalInvocationFailure, WorthQueryTemporalInvocationFailureKind,
};

/// How Query advances the authoritative temporal-intent record and managed
/// wake in the same atomic commit as [`WorthQueryTemporalOperationInvoker::apply`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalOccurrenceOutcome<Input> {
    /// Mark the intent completed (`completed_lifecycle`) and retire its managed wake.
    Complete,
    /// Keep the intent active (`active_lifecycle`) and reschedule its next occurrence
    /// at `next_due_nanoseconds` (strictly greater than the current due coordinate).
    Reschedule {
        next_due_nanoseconds: u64,
        next_input: Input,
        next_input_identity: WorthQueryTemporalOperationInputIdentity,
        next_idempotency: WorthQueryTemporalIntentIdempotencyRelation,
    },
}

/// Application behavior invoked only after Query has freshly admitted the
/// exact installed operation. The invoker describes decision reads and domain
/// effects; Query owns temporal-intent advancement and the final commit.
pub trait WorthQueryTemporalOperationInvoker<Schema, Operation, Input, Scope>:
    Send + Sync + 'static
{
    const SEMANTIC_IDENTITY: &'static str;
    type Projection: 'static;

    fn preconditions(
        &self,
        _input: &Input,
    ) -> worth_query_declaration::facade::application_schema::TypedMutationPreconditions<
        Schema,
        Operation,
        Scope,
    > {
        Default::default()
    }

    fn project(
        &self,
        input: &Input,
        reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
            '_,
            '_,
            Schema,
            Operation,
        >,
        scope: &WorthQueryInvariantEntityIdentity<Schema, Scope>,
    ) -> Result<Self::Projection, WorthQueryTemporalInvocationFailure>;

    /// Selects whether the firing occurrence completes the intent or reschedules
    /// its next recurring occurrence atomically in the same commit.
    ///
    /// Defaults to [`WorthQueryTemporalOccurrenceOutcome::Complete`]. Returning
    /// [`WorthQueryTemporalOccurrenceOutcome::Reschedule`] requires a due field
    /// configured on `WorthQueryTemporalOperationExecution::with_due_field` and
    /// a strictly increasing `next_due_nanoseconds`.
    fn next_occurrence(
        &self,
        _input: &Input,
        _projection: &Self::Projection,
        _current_due_nanoseconds: u64,
        _next_revision: u64,
    ) -> Result<WorthQueryTemporalOccurrenceOutcome<Input>, WorthQueryTemporalInvocationFailure>
    {
        Ok(WorthQueryTemporalOccurrenceOutcome::Complete)
    }

    fn apply(
        &self,
        input: Input,
        projection: Self::Projection,
        effects: &mut WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryTemporalInvocationFailure>;
}
