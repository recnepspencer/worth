use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;

mod application;
mod attempt;
use attempt::ConditionalExecutionProviders;
mod preparation;
mod unwind;
pub(crate) mod work;
pub(crate) use unwind::SignalConditionalAttemptOutcome;

use super::{
    InstalledSignalConditionResolver, InstalledSignalConditionalContract,
    SignalConditionalDecisionCounters, SignalConditionalDecisionEvidence,
};

pub struct SignalConditionalExecutionRequest<'a> {
    pub(super) contract: &'a InstalledSignalConditionalContract,
    pub(super) snapshot_identity: &'a str,
    pub(super) execution_identity: &'a str,
    pub(super) attempt: u64,
    pub(super) force_on_demand: bool,
}

#[derive(Debug)]
pub struct SignalConditionalExecutionFailure {
    error: SignalError,
    counters: SignalConditionalDecisionCounters,
}

impl SignalConditionalExecutionFailure {
    pub const fn counters(&self) -> SignalConditionalDecisionCounters {
        self.counters
    }

    pub fn into_error(self) -> SignalError {
        self.error
    }
}

impl<'a> SignalConditionalExecutionRequest<'a> {
    pub fn new(
        contract: &'a InstalledSignalConditionalContract,
        snapshot_identity: &'a str,
        execution_identity: &'a str,
        attempt: u64,
    ) -> Self {
        Self {
            contract,
            snapshot_identity,
            execution_identity,
            attempt,
            force_on_demand: false,
        }
    }

    pub fn force_on_demand(mut self) -> Self {
        self.force_on_demand = true;
        self
    }

    pub(crate) fn contract(&self) -> &InstalledSignalConditionalContract {
        self.contract
    }
}

impl SignalGraph {
    pub fn execute_installed_conditional(
        &mut self,
        execution_request: worth_execution::ExecutionRequest<'_, '_>,
        request: SignalConditionalExecutionRequest<'_>,
        condition_resolver: &mut impl InstalledSignalConditionResolver,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure> {
        let mut attempt = Some((request, compute));
        let maximum_visits = self
            .installed_runtime_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        let outcome = crate::logic::planner::run_signal_preparation_request(
            execution_request, |request_work, _, _| {
                let (request, compute) = attempt.take().expect("one conditional request step");
                let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(maximum_visits);
                let mut checkpoint = |units: usize| request_work.checkpoint(units as u64)
                    .map_err(|stop| crate::data::retained_storage::RetainedStoragePreparationDenial::ExecutionStopped(stop.into()));
                let mut work = work.reborrow_with_checkpoint(&mut checkpoint);
                Ok(self.execute_installed_conditional_attempt(
                    request, condition_resolver, comparator_resolver, compute, &mut work,
                ))
            },
        ).map_err(|error| SignalConditionalExecutionFailure {
            error, counters: SignalConditionalDecisionCounters::default(),
        })?;
        outcome.0.resume()
    }
}
