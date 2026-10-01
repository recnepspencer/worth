//! Direct candidate lookup for ordinary or standalone cause preparation.
//! Checked graph epochs carry a prepared candidate map through publication.

use worth_execution::{ExecutionResourceLease, MapKernelContext};

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::evaluation::EvaluationWork;

use super::discovery::query_reverse_subscriptions_readonly;
use super::ReverseSubscriptionQuery;

impl SignalGraph {
    pub(crate) fn collect_reverse_subscription_queries(
        &mut self,
        delta: &ProducedAspectDelta,
        work: &mut EvaluationWork<'_, '_>,
        lease: Option<&ExecutionResourceLease<'_>>,
        mut request_work: Option<&mut MapKernelContext<'_, '_>>,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<Vec<ReverseSubscriptionQuery>, SignalError> {
        if !self.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        if lease.is_some() && request_work.is_none() {
            return Err(SignalError::invalid_input(
                "leased candidate lookup requires request work",
            ));
        }
        let changes = delta.changes.as_slice();
        if let Some(budget) = preparation {
            budget.claim_vec::<ReverseSubscriptionQuery>(changes.len())?;
        }
        let mut queries = Vec::with_capacity(changes.len());
        for change in changes {
            let mut query = if let Some(request) = request_work.as_deref_mut() {
                let mut checkpoint = |units: usize| {
                    request
                        .checkpoint(units as u64)
                        .map_err(|_| SignalError::invalid_input("candidate lookup work stopped"))
                };
                query_reverse_subscriptions_readonly(
                    &self.topology.reverse_subscriptions,
                    self.observation.partition_interner(),
                    delta.producer,
                    change,
                    delta.scope_precision,
                    &mut EvaluationWork::RequestCheckpoint(&mut checkpoint),
                )?
            } else {
                query_reverse_subscriptions_readonly(
                    &self.topology.reverse_subscriptions,
                    self.observation.partition_interner(),
                    delta.producer,
                    change,
                    delta.scope_precision,
                    work,
                )?
            };
            work.reserve(Some(query.candidates.len()))?;
            self.with_telemetry(|telemetry| {
                telemetry.invalidation.reverse_subscription_bucket_probes += query.bucket_probes;
                telemetry
                    .invalidation
                    .reverse_subscription_candidates_returned += query.candidates.len() as u64;
            });
            query
                .candidates
                .retain(|candidate| self.is_alive(*candidate));
            queries.push(query);
        }
        Ok(queries)
    }
}
