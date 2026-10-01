//! Fold an epoch's producer changes into one replacement per affected consumer.
use std::collections::BTreeMap;

use super::{PreparedConsumerCauseSet, PreparedDirectCounterDeltas};
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::binding::OutputCommitOrdinal;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::evaluation::EvaluationWork;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

fn with_epoch_cause_work<R>(
    request: Option<&mut MapKernelContext<'_, '_>>,
    fallback: &mut EvaluationWork<'_, '_>,
    operation: impl FnOnce(&mut EvaluationWork<'_, '_>) -> Result<R, SignalError>,
) -> Result<R, SignalError> {
    if let Some(request) = request {
        let mut checkpoint = |units: usize| {
            let units = u64::try_from(units)
                .map_err(|_| SignalError::invalid_input("cause copy work overflow"))?;
            request
                .checkpoint(units)
                .map_err(|_| SignalError::invalid_input("cause copy work stopped"))
        };
        operation(&mut EvaluationWork::RequestCheckpoint(&mut checkpoint))
    } else {
        operation(fallback)
    }
}

pub(crate) struct PreparedEpochDirectCauseAdmission {
    pub(super) commits: Vec<ProducedAspectDelta>,
    pub(super) replacements: Vec<PreparedConsumerCauseSet>,
    pub(super) transitions: BTreeMap<(OutputCommitOrdinal, NodeId), bool>,
    pub(super) counter_deltas: PreparedDirectCounterDeltas,
}

/// The index only identifies rows. Cause payloads have one owned Vec backing;
/// overlay reads borrow it and finalization sorts that same backing in place.
pub(super) struct EpochCauseRows {
    index: BTreeMap<NodeId, usize>,
    rows: Vec<PreparedConsumerCauseSet>,
}

impl EpochCauseRows {
    fn new() -> Self {
        Self {
            index: BTreeMap::new(),
            rows: Vec::new(),
        }
    }

    pub(super) fn causes(
        &self,
        consumer: NodeId,
    ) -> Option<&[crate::data::proof::invalidation::binding::ResolvedDependencyCause]> {
        self.index
            .get(&consumer)
            .map(|&position| &self.rows[position].causes[..])
    }

    pub(super) fn lookup_steps(&self) -> usize {
        crate::data::retained_storage::std_btree_lookup_steps(self.index.len())
    }

    fn insert(
        &mut self,
        replacement: PreparedConsumerCauseSet,
        mut preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<(), SignalError> {
        if let Some(&position) = self.index.get(&replacement.consumer) {
            self.rows[position] = replacement;
            return Ok(());
        }
        if let Some(budget) = preparation.as_deref_mut() {
            let before = crate::data::retained_storage::btree_structure_charge::<NodeId, usize>(
                self.index.len(),
            )
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let after = crate::data::retained_storage::btree_structure_charge::<NodeId, usize>(
                self.index.len() + 1,
            )
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            budget.claim(after.bytes() - before.bytes())?;
        }
        let position = self.rows.len();
        crate::data::request_preparation::push(&mut self.rows, replacement, preparation)?;
        self.index.insert(self.rows[position].consumer, position);
        Ok(())
    }

    fn into_ordered(
        mut self,
        work: &mut EvaluationWork<'_, '_>,
    ) -> Result<Vec<PreparedConsumerCauseSet>, SignalError> {
        // Pop the existing key index in canonical order and permute the one
        // owned row Vec. At most one large inline row swap is needed per key;
        // sorting whole rows would move them across every comparison layer.
        for ordered in 0..self.rows.len() {
            work.reserve(Some(self.lookup_steps()))?;
            let (consumer, position) = self
                .index
                .pop_first()
                .ok_or_else(|| SignalError::internal("epoch cause index lost a replacement"))?;
            if position == ordered {
                continue;
            }
            let displaced = self.rows[ordered].consumer;
            let move_work = std::mem::size_of::<PreparedConsumerCauseSet>()
                .checked_mul(2)
                .and_then(|bytes| bytes.checked_add(self.lookup_steps()))
                .ok_or_else(|| SignalError::invalid_input("epoch cause row move overflow"))?;
            work.reserve(Some(move_work))?;
            self.rows.swap(ordered, position);
            *self.index.get_mut(&displaced).ok_or_else(|| {
                SignalError::internal("epoch cause displaced row lost its index")
            })? = position;
            debug_assert_eq!(self.rows[ordered].consumer, consumer);
        }
        Ok(self.rows)
    }
}

/// Canonical producer position retained while consumer causes fold cumulatively.
pub(crate) struct EpochCauseHead {
    pub(crate) producer: NodeId,
    pub(crate) clean: bool,
    pub(crate) delta_ordinal: Option<OutputCommitOrdinal>,
}

impl SignalGraph {
    pub(crate) fn prepare_epoch_direct_output_causes(
        &mut self,
        deltas: &[ProducedAspectDelta],
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        work: &mut EvaluationWork<'_, '_>,
        lease: Option<&ExecutionResourceLease<'_>>,
        mut candidate_queries: Option<&mut crate::data::graph::PreparedCandidateQueries>,
        mut request_work: Option<&mut MapKernelContext<'_, '_>>,
        mut preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedEpochDirectCauseAdmission, SignalError> {
        let mut replacements = EpochCauseRows::new();
        let mut transitions = BTreeMap::new();
        let mut counter_deltas = PreparedDirectCounterDeltas::default();
        for delta in deltas {
            let queries = if let Some(candidates) = candidate_queries.as_deref_mut() {
                // The checked candidate owner claimed one additional query
                // slot per selected aspect before any evaluator callback.
                let mut prepared = Vec::with_capacity(delta.changes.as_slice().len());
                for change in delta.changes.as_slice() {
                    let mut query = candidates.take(delta.producer, change.aspect)?;
                    if let Some(request) = request_work.as_deref_mut() {
                        request
                            .checkpoint(query.candidates.len() as u64)
                            .map_err(|_| {
                                SignalError::invalid_input("candidate filtering work stopped")
                            })?;
                    }
                    self.with_telemetry(|telemetry| {
                        telemetry.invalidation.reverse_subscription_bucket_probes +=
                            query.bucket_probes;
                        telemetry
                            .invalidation
                            .reverse_subscription_candidates_returned +=
                            query.candidates.len() as u64;
                    });
                    query
                        .candidates
                        .retain(|candidate| self.is_alive(*candidate));
                    prepared.push(query);
                }
                Some(prepared)
            } else if lease.is_some() {
                Some(self.collect_reverse_subscription_queries(
                    delta,
                    &mut EvaluationWork::Ordinary,
                    lease,
                    request_work.as_deref_mut(),
                    preparation.as_deref_mut(),
                )?)
            } else {
                None
            };
            let admission = if let Some(request) = request_work.as_deref_mut() {
                let mut checkpoint = |units: usize| {
                    let units = u64::try_from(units)
                        .map_err(|_| SignalError::invalid_input("cause work size overflow"))?;
                    request
                        .checkpoint(units)
                        .map_err(|_| SignalError::invalid_input("cause preparation work stopped"))
                };
                self.prepare_direct_output_causes_with_overlay(
                    delta,
                    comparator_resolver,
                    &mut EvaluationWork::RequestCheckpoint(&mut checkpoint),
                    None,
                    None,
                    Some(&replacements),
                    preparation.as_deref_mut(),
                    queries,
                )?
            } else {
                self.prepare_direct_output_causes_with_overlay(
                    delta,
                    comparator_resolver,
                    work,
                    None,
                    None,
                    Some(&replacements),
                    preparation.as_deref_mut(),
                    queries,
                )?
            };
            counter_deltas.merge(admission.counter_deltas);
            for replacement in admission.replacements {
                let lookup_steps = replacements.lookup_steps();
                with_epoch_cause_work(request_work.as_deref_mut(), work, |work| {
                    work.reserve(lookup_steps.checked_mul(2).and_then(|n| n.checked_add(1)))
                })?;
                if let Some(budget) = preparation.as_deref_mut() {
                    let before = crate::data::retained_storage::btree_structure_charge::<
                        (OutputCommitOrdinal, NodeId),
                        bool,
                    >(transitions.len())
                    .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
                    let after = crate::data::retained_storage::btree_structure_charge::<
                        (OutputCommitOrdinal, NodeId),
                        bool,
                    >(transitions.len() + 1)
                    .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
                    budget.claim(after.bytes() - before.bytes())?;
                }
                transitions.insert(
                    (delta.output_commit_ordinal, replacement.consumer),
                    replacement.causes.is_empty(),
                );
                replacements.insert(replacement, preparation.as_deref_mut())?;
            }
        }
        with_epoch_cause_work(request_work.as_deref_mut(), work, |work| {
            work.reserve(Some(replacements.rows.len()))?;
            for delta in deltas {
                super::preparation_work::admit_delta_copy(delta, work)?;
            }
            Ok(())
        })?;
        if let Some(budget) = preparation.as_deref_mut() {
            budget.claim_vec::<ProducedAspectDelta>(deltas.len())?;
        }
        for delta in deltas {
            super::preparation_work::claim_delta_copy(delta, preparation.as_deref_mut())?;
        }
        let replacements =
            with_epoch_cause_work(request_work, work, |work| replacements.into_ordered(work))?;
        Ok(PreparedEpochDirectCauseAdmission {
            commits: deltas.to_vec(),
            replacements,
            transitions,
            counter_deltas,
        })
    }
}
