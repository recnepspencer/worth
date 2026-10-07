//! Direct candidate lookup for ordinary or standalone cause preparation.
//! Checked graph epochs carry a prepared candidate map through publication.

use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::logic::evaluation::EvaluationWork;

use super::discovery::query_reverse_subscriptions_readonly;
use super::ReverseSubscriptionQuery;

impl SignalGraph {
    pub(crate) fn collect_reverse_subscription_queries(
        &mut self,
        delta: &ProducedAspectDelta,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<Vec<ReverseSubscriptionQuery>, SignalError> {
        if !self.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        let changes = delta.changes.as_slice();
        work.claim_preparation_vec::<ReverseSubscriptionQuery>(changes.len())?;
        let mut queries = Vec::with_capacity(changes.len());
        for change in changes {
            let mut query = query_reverse_subscriptions_readonly(
                &self.topology.reverse_subscriptions,
                self.observation.partition_interner(),
                delta.producer,
                change,
                delta.scope_precision,
                work,
            )?;
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
