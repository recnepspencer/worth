//! The owner holds completion while sealed requests carry only prepared compute.
use super::HandlerResult;
use crate::domain_computation::primary_graph::application_contribution::{
    computation::prepared_request::{ComputedSlot, PreparedCompute},
    QueryRequestExecution,
};
use std::any::Any;

/// One handler's prepared computation requests and owner-only completion.
///
/// Noncomputing handlers use an immediate plan. Computing handlers prepare all
/// reads before returning their plan; completion is run by the owner after the
/// requests finish. Author-held data may contain handles: their use from a
/// worker follows the worker reentry rule.
#[must_use]
pub struct WorthQueryDecisionPlan<Value, Denial> {
    requests: Vec<PreparedCompute>,
    complete: Box<dyn FnOnce(Vec<ComputedSlot>) -> HandlerResult<Value, Denial>>,
}

impl<Value: 'static, Denial: 'static> WorthQueryDecisionPlan<Value, Denial> {
    pub(in crate::domain_computation::primary_graph) fn has_compute(&self) -> bool {
        !self.requests.is_empty()
    }
    pub fn immediate(result: HandlerResult<Value, Denial>) -> Self {
        Self {
            requests: Vec::new(),
            complete: Box::new(move |slots| {
                assert!(slots.is_empty(), "an immediate plan has no compute slots");
                result
            }),
        }
    }

    /// Transforms a successful decision on the owner. This callback never runs
    /// on a compute worker and may retain owner-only completion data.
    pub fn map<Next: 'static>(
        self,
        map: impl FnOnce(Value) -> Next + 'static,
    ) -> WorthQueryDecisionPlan<Next, Denial> {
        self.and_then(move |value| HandlerResult::Completed(map(value)))
    }

    /// Completes a successful decision on the owner, including a caught denial.
    /// The callback is owner-only and can carry data that is not `Send`.
    pub fn and_then<Next: 'static>(
        self,
        complete_value: impl FnOnce(Value) -> HandlerResult<Next, Denial> + 'static,
    ) -> WorthQueryDecisionPlan<Next, Denial> {
        let Self { requests, complete } = self;
        WorthQueryDecisionPlan {
            requests,
            complete: Box::new(move |slots| match complete(slots) {
                HandlerResult::Completed(value) => complete_value(value),
                HandlerResult::DomainDenied(denial) => HandlerResult::DomainDenied(denial),
                HandlerResult::ExecutionDenied(denial) => HandlerResult::ExecutionDenied(denial),
                HandlerResult::Cancelled => HandlerResult::Cancelled,
                HandlerResult::DeadlineExceeded => HandlerResult::DeadlineExceeded,
            }),
        }
    }

    /// Appends another prepared computation in the same member. Both preparations
    /// precede dispatch; compute requests and owner completions retain their order.
    pub fn zip<Other: 'static>(
        self,
        other: WorthQueryDecisionPlan<Other, Denial>,
    ) -> WorthQueryDecisionPlan<(Value, Other), Denial> {
        let Self {
            mut requests,
            complete,
        } = self;
        let WorthQueryDecisionPlan {
            requests: next_requests,
            complete: next_complete,
        } = other;
        let split = requests.len();
        requests.extend(next_requests);
        WorthQueryDecisionPlan {
            requests,
            complete: Box::new(move |mut slots| {
                let next_slots = slots.split_off(split);
                match complete(slots) {
                    HandlerResult::Completed(value) => match next_complete(next_slots) {
                        HandlerResult::Completed(next) => HandlerResult::Completed((value, next)),
                        HandlerResult::DomainDenied(cause) => HandlerResult::DomainDenied(cause),
                        HandlerResult::ExecutionDenied(cause) => {
                            HandlerResult::ExecutionDenied(cause)
                        }
                        HandlerResult::Cancelled => HandlerResult::Cancelled,
                        HandlerResult::DeadlineExceeded => HandlerResult::DeadlineExceeded,
                    },
                    HandlerResult::DomainDenied(cause) => HandlerResult::DomainDenied(cause),
                    HandlerResult::ExecutionDenied(cause) => HandlerResult::ExecutionDenied(cause),
                    HandlerResult::Cancelled => HandlerResult::Cancelled,
                    HandlerResult::DeadlineExceeded => HandlerResult::DeadlineExceeded,
                }
            }),
        }
    }

    /// Captures a fixture's existing compute probes on the computing thread and
    /// delivers them to its owner-only completion. No preparation loan is lent.
    #[cfg(feature = "test-query-execution-observer")]
    pub fn observe_computes_for_test<Observation: Send + 'static>(
        self,
        before: impl Fn() + Send + Sync + 'static,
        after: impl Fn() -> Observation + Send + Sync + 'static,
        observed: impl FnOnce(Vec<Observation>) + 'static,
    ) -> Self {
        let Self { requests, complete } = self;
        let before = std::sync::Arc::new(before);
        let after = std::sync::Arc::new(after);
        let requests = requests
            .into_iter()
            .map(|request| {
                let before = before.clone();
                let after = after.clone();
                PreparedCompute::new(move |member| {
                    before();
                    let slot = request.run(member);
                    (slot, after())
                })
            })
            .collect();
        Self {
            requests,
            complete: Box::new(move |slots| {
                let (slots, observations): (Vec<_>, Vec<_>) = slots
                    .into_iter()
                    .map(|slot| slot.into_value::<(ComputedSlot, Observation)>())
                    .unzip();
                observed(observations);
                complete(slots)
            }),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn prepared<T: Any + Send>(
        request: PreparedCompute,
        complete: impl FnOnce(T) -> HandlerResult<Value, Denial> + 'static,
    ) -> Self {
        Self {
            requests: vec![request],
            complete: Box::new(move |mut slots| {
                assert_eq!(slots.len(), 1, "one sealed request has one computed slot");
                let value = slots
                    .pop()
                    .expect("the aligned computed slot")
                    .into_value::<T>();
                complete(value)
            }),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn run_serial(
        self,
        execution: &QueryRequestExecution<'_>,
    ) -> HandlerResult<Value, Denial> {
        let (requests, completion) = self.into_parts();
        let slots = requests
            .into_iter()
            .map(|request| {
                request.run(
                    super::super::super::application_contribution::WorthQueryMemberExecution::new(
                        execution,
                    ),
                )
            })
            .collect();
        completion.finish(slots)
    }

    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (Vec<PreparedCompute>, DecisionCompletion<Value, Denial>) {
        let Self { requests, complete } = self;
        (requests, DecisionCompletion { complete })
    }
}

/// This continuation stays on the owner; only its aligned requests travel.
pub(in crate::domain_computation::primary_graph) struct DecisionCompletion<Value, Denial> {
    complete: Box<dyn FnOnce(Vec<ComputedSlot>) -> HandlerResult<Value, Denial>>,
}

impl<Value, Denial> DecisionCompletion<Value, Denial> {
    pub(in crate::domain_computation::primary_graph) fn finish(
        self,
        slots: Vec<ComputedSlot>,
    ) -> HandlerResult<Value, Denial> {
        (self.complete)(slots)
    }
}
