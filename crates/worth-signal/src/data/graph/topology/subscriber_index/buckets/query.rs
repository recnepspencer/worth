use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::graph::subscription_candidates;
use crate::data::handle::NodeId;
use crate::data::output::{InternedPartitionSubscription, PartitionMatchMode};
use crate::data::retained_storage::ordered_lookup_steps;
use crate::logic::evaluation::EvaluationWork;

use super::fork_overlay::{
    extend_merged_set, BucketDelta, ReverseSubscriptionStorage, SetMergeTraversal,
};
use super::{
    DetailScopeKey, ProducerAspectKey, ReverseSubscriptionIndex, ReverseSubscriptionQuery,
    SubscriberScopeBuckets,
};

impl ReverseSubscriptionIndex {
    pub(crate) fn query_whole_aspect(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut EvaluationWork<'_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        self.query_whole_aspect_observed(producer, aspect, work)
            .map(|(query, _)| query)
    }

    fn query_whole_aspect_observed(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(ReverseSubscriptionQuery, SetMergeTraversal), SignalError> {
        let key = ProducerAspectKey::from_committed_output(producer, aspect);
        let (base, delta) = self.bucket_view(&key, work)?;
        if base.is_none() && delta.is_none() {
            return Ok((empty_query(1), SetMergeTraversal::default()));
        }
        let mut candidates = Vec::new();
        let traversal = extend_merged_set(
            base.map(|buckets| &buckets.all),
            delta.map(|delta| &delta.all),
            &mut candidates,
            work,
        )?;
        Ok((finish_query(candidates, 1, work)?, traversal))
    }

    #[cfg(test)]
    pub(super) fn query_whole_aspect_with_traversal(
        &self,
        producer: NodeId,
        aspect: Aspect,
    ) -> (ReverseSubscriptionQuery, SetMergeTraversal) {
        self.query_whole_aspect_observed(producer, aspect, &mut EvaluationWork::Ordinary)
            .unwrap()
    }

    pub(super) fn query_unscoped(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut EvaluationWork<'_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        let key = ProducerAspectKey::from_committed_output(producer, aspect);
        let (base, delta) = self.bucket_view(&key, work)?;
        if base.is_none() && delta.is_none() {
            return Ok(empty_query(1));
        }
        let mut candidates = Vec::new();
        let _ = extend_merged_set(
            base.map(|buckets| &buckets.unscoped),
            delta.map(|delta| &delta.unscoped),
            &mut candidates,
            work,
        )?;
        finish_query(candidates, 1, work)
    }

    pub(crate) fn query_scope(
        &self,
        producer: NodeId,
        aspect: Aspect,
        scope: InternedPartitionSubscription,
        work: &mut EvaluationWork<'_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        let key = ProducerAspectKey::from_committed_output(producer, aspect);
        let (base, delta) = self.bucket_view(&key, work)?;
        if base.is_none() && delta.is_none() {
            return Ok(empty_query(match scope.match_mode {
                PartitionMatchMode::WholePartition => 2,
                PartitionMatchMode::PartitionAndDetail => 3,
            }));
        }
        let mut candidates = Vec::new();
        let _ = extend_merged_set(
            base.map(|buckets| &buckets.unscoped),
            delta.map(|delta| &delta.unscoped),
            &mut candidates,
            work,
        )?;
        let probes = match scope.match_mode {
            PartitionMatchMode::WholePartition => {
                admit_scope_lookup(
                    base.map_or(0, |b| b.partition_scoped.len()),
                    delta.map_or(0, |d| d.partition_scoped.len()),
                    work,
                )?;
                let _ = extend_merged_set(
                    base.and_then(|buckets| buckets.partition_scoped.get(&scope.partition)),
                    delta.and_then(|delta| delta.partition_scoped.get(&scope.partition)),
                    &mut candidates,
                    work,
                )?;
                2
            }
            PartitionMatchMode::PartitionAndDetail => {
                admit_scope_lookup(
                    base.map_or(0, |b| b.whole_partitions.len()),
                    delta.map_or(0, |d| d.whole_partitions.len()),
                    work,
                )?;
                let _ = extend_merged_set(
                    base.and_then(|buckets| buckets.whole_partitions.get(&scope.partition)),
                    delta.and_then(|delta| delta.whole_partitions.get(&scope.partition)),
                    &mut candidates,
                    work,
                )?;
                if let Some(detail) = scope.detail {
                    admit_scope_lookup(
                        base.map_or(0, |b| b.exact_details.len()),
                        delta.map_or(0, |d| d.exact_details.len()),
                        work,
                    )?;
                    let key = DetailScopeKey {
                        partition: scope.partition,
                        detail,
                    };
                    let _ = extend_merged_set(
                        base.and_then(|buckets| buckets.exact_details.get(&key)),
                        delta.and_then(|delta| delta.exact_details.get(&key)),
                        &mut candidates,
                        work,
                    )?;
                }
                3
            }
        };
        finish_query(candidates, probes, work)
    }

    fn bucket_view(
        &self,
        key: &ProducerAspectKey,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(Option<&SubscriberScopeBuckets>, Option<&BucketDelta>), SignalError> {
        Ok(match &self.storage {
            ReverseSubscriptionStorage::Exclusive(flat) => {
                work.reserve(Some(4 * ordered_lookup_steps(flat.buckets.len())))?;
                (flat.buckets.get(key), None)
            }
            ReverseSubscriptionStorage::ForkShared {
                base,
                bucket_changes,
                ..
            } => {
                admit_scope_lookup(base.buckets.len(), bucket_changes.len(), work)?;
                (base.buckets.get(key), bucket_changes.get(key))
            }
        })
    }
}

fn empty_query(bucket_probes: u64) -> ReverseSubscriptionQuery {
    ReverseSubscriptionQuery {
        candidates: Vec::new(),
        bucket_probes,
    }
}

fn finish_query(
    mut candidates: Vec<NodeId>,
    bucket_probes: u64,
    work: &mut EvaluationWork<'_>,
) -> Result<ReverseSubscriptionQuery, SignalError> {
    subscription_candidates::normalize(&mut candidates, work)?;
    Ok(ReverseSubscriptionQuery {
        candidates,
        bucket_probes,
    })
}

fn admit_scope_lookup(
    base_len: usize,
    delta_len: usize,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    // Producer/aspect and partition/detail keys contain at most three words.
    work.reserve(Some(
        4 * (ordered_lookup_steps(base_len) + ordered_lookup_steps(delta_len)),
    ))
}
