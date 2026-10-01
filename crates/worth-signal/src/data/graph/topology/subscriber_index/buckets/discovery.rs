use super::ReverseSubscriptionQuery;
use crate::data::error::SignalError;
use crate::data::graph::{subscription_candidates, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::output::PartitionInterner;
use crate::data::proof::invalidation::output_commit::{ProducedAspectChange, ScopePrecision};
use crate::logic::evaluation::EvaluationWork;

impl SignalGraph {
    pub(crate) fn query_reverse_subscriptions(
        &mut self,
        producer: NodeId,
        change: &ProducedAspectChange,
        precision: ScopePrecision,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        if !self.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        let mut result = query_reverse_subscriptions_readonly(
            &self.topology.reverse_subscriptions,
            self.observation.partition_interner(),
            producer,
            change,
            precision,
            work,
        )?;
        work.reserve(Some(result.candidates.len()))?;
        if let Some(mut telemetry) = self.telemetry_mut() {
            telemetry.invalidation.reverse_subscription_bucket_probes += result.bucket_probes;
            telemetry
                .invalidation
                .reverse_subscription_candidates_returned += result.candidates.len() as u64;
        }
        result
            .candidates
            .retain(|candidate| self.is_alive(*candidate));
        Ok(result)
    }
}

pub(super) fn query_reverse_subscriptions_readonly(
    index: &super::ReverseSubscriptionIndex,
    interner: &PartitionInterner,
    producer: NodeId,
    change: &ProducedAspectChange,
    precision: ScopePrecision,
    work: &mut EvaluationWork<'_, '_>,
) -> Result<ReverseSubscriptionQuery, SignalError> {
    if precision == ScopePrecision::ConservativeLegacyUnion || change.changed_scopes.is_empty() {
        index.query_whole_aspect(producer, change.aspect, work)
    } else {
        let mut candidates = Vec::new();
        let mut bucket_probes = 0;
        work.reserve(Some(change.changed_scopes.len()))?;
        for scope in change.changed_scopes.as_slice() {
            work.reserve(interner.subscription_lookup_work(scope))?;
            let interned = interner.resolve_subscription(scope);
            let Some(interned) = interned else {
                let known = interner.resolve_known_prefix(scope.path());
                if let Some(known) = known {
                    let query = index.query_known_scope(
                        producer,
                        change.aspect,
                        known,
                        scope.coverage(),
                        false,
                        work,
                    )?;
                    bucket_probes += query.bucket_probes;
                    subscription_candidates::append(&mut candidates, query.candidates, work)?;
                    continue;
                }
                let query = index.query_unscoped(producer, change.aspect, work)?;
                bucket_probes += query.bucket_probes;
                subscription_candidates::append(&mut candidates, query.candidates, work)?;
                continue;
            };
            let query = index.query_scope(producer, change.aspect, interned, work)?;
            bucket_probes += query.bucket_probes;
            subscription_candidates::append(&mut candidates, query.candidates, work)?;
        }
        subscription_candidates::normalize(&mut candidates, work)?;
        Ok(ReverseSubscriptionQuery {
            candidates,
            bucket_probes,
        })
    }
}
