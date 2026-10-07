//! A selected epoch's candidate lookup map is admitted before its evaluators.
//! Canonical deltas later select which of its finite aspect slots do work.

use worth_execution::{
    ChargedBytes, ExecutionMap, ExecutionResourceLease, LeaseRequest, MapKernelContext,
    MapKernelFailure, MapOutcome, MapPartition, PreparedExecutionMap,
};
use worth_foundational::{ExecutionPosture, ExecutionRequestPolicy, PartitionIdentity};

use crate::data::aspect::{Aspect, AspectMask, MAX_ASPECTS};
use crate::data::error::{SignalError, SignalExecutionStop, SignalPublicationDisposition};
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::output_commit::{
    ProducedAspectChange, ProducedAspectDelta, ScopePrecision,
};
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::evaluation::EvaluationWork;

use super::discovery::query_reverse_subscriptions_readonly;
use super::ReverseSubscriptionQuery;

#[derive(Clone, Copy)]
pub(super) struct CandidateTask(usize);

impl ChargedBytes for CandidateTask {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl ChargedBytes for ReverseSubscriptionQuery {
    fn additional_charged_bytes(&self) -> u64 {
        self.candidates
            .capacity()
            .saturating_mul(std::mem::size_of::<NodeId>()) as u64
    }
}

type CandidateMap<'authority> =
    PreparedExecutionMap<'authority, CandidateTask, u8, ReverseSubscriptionQuery, SignalError>;

pub(crate) struct PreparedCandidateEpoch<'authority> {
    map: CandidateMap<'authority>,
    keys: Vec<(NodeId, Aspect)>,
    used: Vec<bool>,
}

pub(crate) struct PreparedCandidateQueries {
    keys: Vec<(NodeId, Aspect)>,
    values: Vec<ReverseSubscriptionQuery>,
    used: Vec<bool>,
}

impl SignalGraph {
    pub(crate) fn prepare_candidate_epoch<'authority>(
        &self,
        producers: impl ExactSizeIterator<Item = NodeId> + Clone,
        output_heap_grant: u64,
        posture: ExecutionPosture,
        lease: &ExecutionResourceLease<'authority>,
        work: &mut MapKernelContext<'_, '_>,
        preparation: &mut SignalPreparationBudget,
    ) -> Result<PreparedCandidateEpoch<'authority>, SignalError> {
        if !self.topology.reverse_subscriptions.is_valid() {
            return Err(SignalError::internal(
                "reverse subscription index requires authority rebuild",
            ));
        }
        let mut key_count = 0_usize;
        for producer in producers.clone() {
            work.checkpoint(MAX_ASPECTS as u64)
                .map_err(SignalError::execution_checkpoint_stopped)?;
            let produces = self.get_contract(producer)?.semantics.produces;
            for index in 0..MAX_ASPECTS {
                let aspect = Aspect::new(index as u8);
                if produces.contains(AspectMask::from_aspect(aspect)) {
                    key_count = key_count.checked_add(1).ok_or_else(|| {
                        SignalError::invalid_input("candidate aspect count overflow")
                    })?;
                }
            }
        }
        preparation.claim_vec::<(NodeId, Aspect)>(key_count)?;
        let mut keys = Vec::with_capacity(key_count);
        for producer in producers {
            work.checkpoint(MAX_ASPECTS as u64)
                .map_err(SignalError::execution_checkpoint_stopped)?;
            let produces = self.get_contract(producer)?.semantics.produces;
            for index in 0..MAX_ASPECTS {
                let aspect = Aspect::new(index as u8);
                if produces.contains(AspectMask::from_aspect(aspect)) {
                    keys.push((producer, aspect));
                }
            }
        }
        work.checkpoint(keys.len() as u64)
            .map_err(SignalError::execution_checkpoint_stopped)?;
        keys.sort_unstable();
        keys.dedup();
        preparation.claim_vec::<MapPartition<CandidateTask, u8>>(keys.len())?;
        preparation.claim_vec::<PartitionIdentity>(keys.len())?;
        preparation.claim_vec::<bool>(keys.len())?;
        preparation.claim_vec::<Option<(&ProducedAspectChange, ScopePrecision)>>(keys.len())?;
        preparation.claim_vec::<ReverseSubscriptionQuery>(keys.len())?;
        preparation.claim_vec::<ReverseSubscriptionQuery>(keys.len())?;
        preparation.claim_vec::<(PartitionIdentity, Vec<u8>)>(keys.len())?;
        preparation.claim_vec::<Vec<u8>>(keys.len())?;
        preparation.claim_vec::<(PartitionIdentity, CandidateTask, u64, u64)>(keys.len())?;
        let mut partitions = Vec::with_capacity(keys.len());
        let mut retained_results = 0_u64;
        for (index, &(producer, aspect)) in keys.iter().enumerate() {
            let capacity = self.topology.reverse_subscriptions.candidate_capacity(
                producer,
                aspect,
                output_heap_grant,
                work,
            )?;
            retained_results = retained_results
                .checked_add(capacity.result_bytes)
                .ok_or_else(|| SignalError::invalid_input("candidate results capacity overflow"))?;
            preparation.claim_vec::<u8>(1)?;
            partitions.push(MapPartition {
                identity: PartitionIdentity::new(index as u64),
                value: CandidateTask(index),
                read_keys: vec![0_u8],
                write_keys: Vec::new(),
                kernel_scratch_bytes: capacity.scratch_bytes,
                max_result_bytes: capacity.result_bytes,
            });
        }
        preparation.claim(retained_results)?;
        let identities = partitions
            .iter()
            .map(|partition| partition.identity)
            .collect();
        let map = ExecutionMap::try_from_declared_partitions(identities, partitions)
            .map_err(|_| SignalError::invalid_input("candidate epoch map declaration denied"))?;
        let child = lease
            .child(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture,
                    lease.policy().determinism(),
                    lease.policy().budget(),
                ),
                deadline: None,
                cancellation: worth_execution::CancellationToken::new(),
            })
            .map_err(SignalError::execution_admission_denied)?;
        let map = map
            .prepare_run(child)
            .map_err(SignalError::execution_admission_denied)?;
        Ok(PreparedCandidateEpoch {
            map,
            used: vec![false; keys.len()],
            keys,
        })
    }
}

impl PreparedCandidateEpoch<'_> {
    pub(crate) fn run(
        self,
        graph: &SignalGraph,
        deltas: &[ProducedAspectDelta],
        work: &mut MapKernelContext<'_, '_>,
    ) -> Result<PreparedCandidateQueries, SignalError> {
        let Self { map, keys, used } = self;
        let mut changed = vec![None; keys.len()];
        for delta in deltas {
            for change in delta.changes.as_slice() {
                work.checkpoint(keys.len().max(1).ilog2() as u64 + 2)
                    .map_err(SignalError::execution_checkpoint_stopped)?;
                let index = keys
                    .binary_search(&(delta.producer, change.aspect))
                    .map_err(|_| {
                        SignalError::internal("changed aspect escaped selected candidate map")
                    })?;
                changed[index] = Some((change, delta.scope_precision));
            }
        }
        let index = &graph.topology.reverse_subscriptions;
        let interner = graph.observation.partition_interner();
        let outcome = map.run(|task, kernel| {
            let Some((change, precision)) = changed[task.0] else {
                return Ok::<_, MapKernelFailure<SignalError>>(ReverseSubscriptionQuery::default());
            };
            let (producer, _) = keys[task.0];
            let mut checkpoint = |units: usize| {
                kernel
                    .checkpoint(units as u64)
                    .map_err(SignalError::execution_checkpoint_stopped)
            };
            let mut query = query_reverse_subscriptions_readonly(
                index,
                interner,
                producer,
                change,
                precision,
                &mut EvaluationWork::RequestCheckpoint(&mut checkpoint),
            )
            .map_err(MapKernelFailure::Domain)?;
            kernel
                .checkpoint(query.candidates.len() as u64)
                .map_err(MapKernelFailure::Stop)?;
            let mut compact = Vec::with_capacity(query.candidates.len());
            compact.extend(query.candidates);
            query.candidates = compact;
            Ok(query)
        });
        let values = match outcome {
            MapOutcome::Complete { values, .. } => values,
            MapOutcome::Stopped {
                boundary,
                reason,
                report,
                ..
            } => {
                return Err(SignalError::execution_stopped(SignalExecutionStop::new(
                    reason.into(),
                    boundary,
                    SignalPublicationDisposition::WorkerLocal,
                    report,
                )));
            }
        };
        Ok(PreparedCandidateQueries { keys, values, used })
    }
}

impl PreparedCandidateQueries {
    pub(crate) fn take(
        &mut self,
        producer: NodeId,
        aspect: Aspect,
    ) -> Result<ReverseSubscriptionQuery, SignalError> {
        let index = self
            .keys
            .binary_search(&(producer, aspect))
            .map_err(|_| SignalError::internal("candidate result lacks a changed aspect"))?;
        if std::mem::replace(&mut self.used[index], true) {
            return Err(SignalError::internal("candidate result consumed twice"));
        }
        Ok(std::mem::take(&mut self.values[index]))
    }
}
