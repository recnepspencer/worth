use super::{HandlerResult, MutationHandlerExecutionDenial};
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork;

/// Projection evidence from this invocation, separate from historical receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMutationHandlerWork {
    /// No decision projection reader executed for this invocation.
    NotStarted,
    Captured(WorthQueryMutationHandlerProjectionWork),
}

/// Captured by the projection owner; descriptive evidence grants no authority.
/// The counters cover decision projection, not candidate construction or commit.
/// Callers cannot manufacture a capture from descriptive counters:
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryInvariantProjectionWork, WorthQueryMutationHandlerProjectionWork,
/// };
/// let _ = WorthQueryMutationHandlerProjectionWork {
///     projection: WorthQueryInvariantProjectionWork::default(),
///     handler_contacted: true,
/// };
/// ```
///
/// Nor can they manufacture a zero-work capture through `Default`:
///
/// ```compile_fail,E0277
/// use worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerProjectionWork;
/// let _: WorthQueryMutationHandlerProjectionWork = Default::default();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryMutationHandlerProjectionWork {
    projection: WorthQueryInvariantProjectionWork,
    handler_contacted: bool,
}

impl WorthQueryMutationHandlerProjectionWork {
    pub const fn projection_work(&self) -> WorthQueryInvariantProjectionWork {
        self.projection
    }

    pub const fn handler_contacted(&self) -> bool {
        self.handler_contacted
    }
}

impl WorthQueryMutationHandlerWork {
    pub(super) const fn captured(
        projection: WorthQueryInvariantProjectionWork,
        handler_contacted: bool,
    ) -> Self {
        Self::Captured(WorthQueryMutationHandlerProjectionWork {
            projection,
            handler_contacted,
        })
    }
}

/// One installed execution's outcome and its owner-captured decision work.
/// Failure and zero-work execution retain their distinct evidence states.
#[derive(Debug)]
pub struct WorthQueryMutationHandlerExecutionReport<Value, Denial> {
    outcome: Result<HandlerResult<Value, Denial>, MutationHandlerExecutionDenial>,
    decision_work: WorthQueryMutationHandlerWork,
}

impl<Value, Denial> WorthQueryMutationHandlerExecutionReport<Value, Denial> {
    pub(super) fn new(
        outcome: Result<HandlerResult<Value, Denial>, MutationHandlerExecutionDenial>,
        decision_work: WorthQueryMutationHandlerWork,
    ) -> Self {
        Self {
            outcome,
            decision_work,
        }
    }

    pub const fn outcome(
        &self,
    ) -> &Result<HandlerResult<Value, Denial>, MutationHandlerExecutionDenial> {
        &self.outcome
    }

    pub const fn decision_work(&self) -> &WorthQueryMutationHandlerWork {
        &self.decision_work
    }

    pub fn into_parts(
        self,
    ) -> (
        Result<HandlerResult<Value, Denial>, MutationHandlerExecutionDenial>,
        WorthQueryMutationHandlerWork,
    ) {
        (self.outcome, self.decision_work)
    }

    pub fn into_outcome(
        self,
    ) -> Result<HandlerResult<Value, Denial>, MutationHandlerExecutionDenial> {
        self.outcome
    }
}
