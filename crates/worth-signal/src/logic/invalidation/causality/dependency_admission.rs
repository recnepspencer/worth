use super::preparation_work;
use crate::data::graph::subscription_candidates;
use crate::logic::evaluation::EvaluationWork;
mod publication;
pub(crate) use publication::{
    PreparedDirectCauseNodes, PreparedDirectCausePublication, PreparedDirectCauseStores,
    PreparedRetainedDirectCauseStores,
};

use crate::data::aspect::AspectMask;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::ResolvedDependencyCause;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;

use super::{changed_scopes_for_edge, reconcile_edge_cause, CauseAdmissionContext};

#[derive(Debug)]
pub(crate) struct PreparedDirectCauseAdmission {
    producer: NodeId,
    commit: Option<ProducedAspectDelta>,
    replacements: Vec<PreparedConsumerCauseSet>,
    counter_deltas: PreparedDirectCounterDeltas,
}

#[derive(Debug)]
struct PreparedConsumerCauseSet {
    consumer: NodeId,
    causes: crate::data::graph::storage::invalidation_causes::NormalizedCauseSet,
    cache: crate::data::graph::storage::PreparedInvalidationCache,
}

enum DirectCandidateAdmission {
    Admitted(PreparedConsumerCauseSet, PreparedDirectCounterDeltas),
    ContractRejected(PreparedDirectCounterDeltas),
    CausalityRejected(PreparedDirectCounterDeltas),
}

#[derive(Debug, Default)]
struct PreparedDirectCounterDeltas {
    source_deltas: u64,
    edges_examined: u64,
    bucket_probes: u64,
    candidates_returned: u64,
    aspect_contract_rejections: u64,
    scope_rejections: u64,
    comparator_rejections: u64,
    settlements: u64,
}

impl PreparedDirectCounterDeltas {
    fn merge(&mut self, other: Self) {
        self.source_deltas += other.source_deltas;
        self.edges_examined += other.edges_examined;
        self.bucket_probes += other.bucket_probes;
        self.candidates_returned += other.candidates_returned;
        self.aspect_contract_rejections += other.aspect_contract_rejections;
        self.scope_rejections += other.scope_rejections;
        self.comparator_rejections += other.comparator_rejections;
        self.settlements += other.settlements;
    }
}

impl SignalGraph {
    pub(crate) fn prepare_direct_output_causes(
        &mut self,
        delta: &ProducedAspectDelta,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        work: &mut EvaluationWork<'_>,
    ) -> Result<PreparedDirectCauseAdmission, SignalError> {
        let mut subscribers = Vec::new();
        let mut counter_deltas = PreparedDirectCounterDeltas {
            source_deltas: 1,
            ..Default::default()
        };
        work.reserve(Some(delta.changes.as_slice().len()))?;
        for change in delta.changes.as_slice() {
            let query = self.query_reverse_subscriptions(
                delta.producer,
                change,
                delta.scope_precision,
                work,
            )?;
            counter_deltas.bucket_probes += query.bucket_probes;
            counter_deltas.candidates_returned += query.candidates.len() as u64;
            let candidate_count = query.candidates.len() as u64;
            self.with_telemetry(|telemetry| {
                telemetry.invalidation.direct_subscriber_candidates_examined += candidate_count;
            });
            subscription_candidates::append(&mut subscribers, query.candidates, work)?;
        }
        subscription_candidates::normalize(&mut subscribers, work)?;
        work.reserve(subscribers.len().checked_mul(2))?;
        let mut replacements = Vec::with_capacity(subscribers.len());
        for &consumer in &subscribers {
            match self.prepare_consumer_cause_set(consumer, delta, comparator_resolver, work)? {
                DirectCandidateAdmission::Admitted(replacement, counters) => {
                    counter_deltas.merge(counters);
                    replacements.push(replacement);
                }
                DirectCandidateAdmission::ContractRejected(counters) => {
                    counter_deltas.merge(counters);
                    self.with_telemetry(|telemetry| {
                        telemetry.invalidation.direct_contract_rejections += 1;
                    });
                }
                DirectCandidateAdmission::CausalityRejected(counters) => {
                    counter_deltas.merge(counters);
                    self.with_telemetry(|telemetry| {
                        telemetry.invalidation.direct_causality_rejections += 1;
                    });
                }
            }
        }
        preparation_work::admit_delta_copy(delta, work)?;
        Ok(PreparedDirectCauseAdmission {
            producer: delta.producer,
            commit: Some(delta.clone()),
            replacements,
            counter_deltas,
        })
    }

    pub(crate) fn prepare_stable_output_resolution(
        &mut self,
        producer: NodeId,
    ) -> Result<PreparedDirectCauseAdmission, SignalError> {
        Ok(PreparedDirectCauseAdmission {
            producer,
            commit: None,
            replacements: Vec::new(),
            counter_deltas: Default::default(),
        })
    }

    fn prepare_consumer_cause_set(
        &self,
        consumer: NodeId,
        delta: &ProducedAspectDelta,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        work: &mut EvaluationWork<'_>,
    ) -> Result<DirectCandidateAdmission, SignalError> {
        let edges = self.current_runtime_dependencies_of(consumer)?;
        // Two filtering passes and each admitted edge's change lookup.
        work.reserve(
            delta
                .changes
                .as_slice()
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_mul(edges.len()))
                .and_then(|n| n.checked_mul(3)),
        )?;
        let relevant = edges.iter().filter(|edge| {
            edge.source() == delta.producer
                && delta
                    .changes
                    .as_slice()
                    .iter()
                    .any(|change| change.aspect == edge.aspect())
        });
        let mut counters = PreparedDirectCounterDeltas {
            edges_examined: relevant.clone().count() as u64,
            ..Default::default()
        };
        if counters.edges_examined == 0 {
            return Ok(DirectCandidateAdmission::CausalityRejected(counters));
        }
        let snapshot = self.get_dep_snapshot(consumer)?;
        let revision = self.dependency_revision(consumer)?;
        let graph_instance = self.runtime_instance_id();
        let config = self.node_eval_config(consumer)?;
        work.reserve(Some(1))?;
        let policy = comparator_resolver.policy_for_node(consumer, config.comparator.as_ref());
        let pending = self.pending_causes(consumer)?;
        preparation_work::admit_causes_copy(pending, work)?;
        let mut causes = pending.to_vec();
        let mut affected = false;
        let mut contract_rejected = false;

        for edge in relevant {
            let Some(change) = delta
                .changes
                .as_slice()
                .iter()
                .find(|change| change.aspect == edge.aspect())
            else {
                continue;
            };
            let Some(changed_scopes) = changed_scopes_for_edge(change, edge.scope_ref(), work)?
            else {
                counters.scope_rejections += 1;
                continue;
            };
            let changed_aspect = AspectMask::from_aspect(change.aspect);
            let contract = self.get_contract(consumer)?;
            work.reserve(
                preparation_work::scope_comparison(edge.scope_ref()).and_then(|cost| {
                    cost.checked_mul(
                        contract
                            .projection
                            .consumes_partitions
                            .as_ref()
                            .map_or(0, Vec::len)
                            .checked_add(2)?,
                    )
                }),
            )?;
            if !contract.cares_about_change(changed_aspect, changed_scopes) {
                contract_rejected = true;
                if contract.cares_about_change(changed_aspect, &[]) {
                    counters.scope_rejections += 1;
                } else {
                    counters.aspect_contract_rejections += 1;
                }
                continue;
            }
            affected = true;
            work.reserve(
                preparation_work::scope_comparison(edge.scope_ref())
                    .and_then(|cost| cost.checked_mul(snapshot.entries().len())),
            )?;
            let Some(cached_version) = snapshot
                .entries()
                .iter()
                .find(|entry| {
                    entry.source == delta.producer
                        && entry.aspect == change.aspect
                        && entry.scope.as_ref() == edge.scope_ref()
                })
                .map(|entry| entry.cached_version)
            else {
                continue;
            };
            work.reserve(Some(1))?;
            let meaningful = policy.has_meaningful_change(
                change.aspect,
                cached_version,
                change.committed_version,
                comparator_resolver,
            )?;
            if meaningful {
                counters.settlements += 1;
            } else {
                counters.comparator_rejections += 1;
            }
            reconcile_edge_cause(
                &mut causes,
                CauseAdmissionContext {
                    graph_instance,
                    consumer,
                    revision,
                    producer: delta.producer,
                    output_commit_ordinal: delta.output_commit_ordinal,
                },
                edge.aspect(),
                edge.scope_ref(),
                cached_version,
                change.committed_version,
                changed_scopes,
                meaningful,
                work,
            )?;
        }
        Ok(if affected {
            let causes =
                crate::data::graph::storage::invalidation_causes::NormalizedCauseSet::prepare(
                    causes, work,
                )?;
            DirectCandidateAdmission::Admitted(
                PreparedConsumerCauseSet {
                    consumer,
                    cache: crate::data::graph::storage::PreparedInvalidationCache::from_causes(
                        &causes, work,
                    )?,
                    causes,
                },
                counters,
            )
        } else if contract_rejected {
            DirectCandidateAdmission::ContractRejected(counters)
        } else {
            DirectCandidateAdmission::CausalityRejected(counters)
        })
    }
}
