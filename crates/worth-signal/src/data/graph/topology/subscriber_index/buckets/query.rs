use crate::data::aspect::Aspect;
use crate::data::error::SignalError;
use crate::data::graph::subscription_candidates;
use crate::data::handle::NodeId;
use crate::data::output::{
    InternedPartitionSubscription, InternedScopePath, ScopeCoverage, ScopePath,
};
use crate::data::retained_storage::ordered_lookup_steps;
use crate::logic::evaluation::EvaluationWork;

use super::fork_overlay::{
    extend_merged_set, BucketDelta, ReverseSubscriptionStorage, SetMergeTraversal,
};
use super::{
    ProducerAspectKey, ReverseSubscriptionIndex, ReverseSubscriptionQuery, SubscriberScopeBuckets,
};

impl ReverseSubscriptionIndex {
    pub(crate) fn query_whole_aspect(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        self.query_whole_aspect_observed(producer, aspect, work)
            .map(|(query, _)| query)
    }

    fn query_whole_aspect_observed(
        &self,
        producer: NodeId,
        aspect: Aspect,
        work: &mut EvaluationWork<'_, '_>,
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
        work: &mut EvaluationWork<'_, '_>,
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
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        self.query_known_scope(producer, aspect, scope.path(), scope.coverage(), true, work)
    }

    pub(crate) fn query_known_scope(
        &self,
        producer: NodeId,
        aspect: Aspect,
        path: InternedScopePath,
        coverage: ScopeCoverage,
        complete: bool,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        let key = ProducerAspectKey::from_committed_output(producer, aspect);
        let (base, delta) = self.bucket_view(&key, work)?;
        let ancestor_depth = if complete {
            path.depth().saturating_sub(1)
        } else {
            path.depth()
        };
        let probes = 1 + ancestor_depth as u64 + u64::from(complete);
        if base.is_none() && delta.is_none() {
            return Ok(empty_query(probes));
        }
        let mut candidates = Vec::new();
        let _ = extend_merged_set(
            base.map(|buckets| &buckets.unscoped),
            delta.map(|delta| &delta.unscoped),
            &mut candidates,
            work,
        )?;
        for depth in 1..=ancestor_depth {
            let prefix = path.prefix(depth).expect("bounded path prefix");
            admit_scope_lookup(
                base.map_or(0, |b| b.subtree_covering.len()),
                delta.map_or(0, |d| d.subtree_covering.len()),
                work,
            )?;
            let _ = extend_merged_set(
                base.and_then(|b| b.subtree_covering.get(&prefix)),
                delta.and_then(|d| d.subtree_covering.get(&prefix)),
                &mut candidates,
                work,
            )?;
        }
        if complete {
            let map = if coverage == ScopeCoverage::Exact {
                ScopeMap::Exact
            } else {
                ScopeMap::Subtree
            };
            let (base_map, delta_map) = match map {
                ScopeMap::Exact => (base.map(|b| &b.same_path), delta.map(|d| &d.same_path)),
                ScopeMap::Subtree => (
                    base.map(|b| &b.subtree_members),
                    delta.map(|d| &d.subtree_members),
                ),
            };
            admit_scope_lookup(
                base_map.map_or(0, |b| b.len()),
                delta_map.map_or(0, |d| d.len()),
                work,
            )?;
            let _ = extend_merged_set(
                base_map.and_then(|b| b.get(&path)),
                delta_map.and_then(|d| d.get(&path)),
                &mut candidates,
                work,
            )?;
        }
        finish_query(candidates, probes, work)
    }

    fn bucket_view(
        &self,
        key: &ProducerAspectKey,
        work: &mut EvaluationWork<'_, '_>,
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

enum ScopeMap {
    Exact,
    Subtree,
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
    work: &mut EvaluationWork<'_, '_>,
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
    work: &mut EvaluationWork<'_, '_>,
) -> Result<(), SignalError> {
    // Each ordered comparison may inspect all eight interned path segments.
    work.reserve(Some(
        ScopePath::MAX_DEPTH * (ordered_lookup_steps(base_len) + ordered_lookup_steps(delta_len)),
    ))
}
