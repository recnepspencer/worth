//! Atomic installation of cumulative retained node evaluation changes.
mod reservation_admission;
use super::NodeArena;
use crate::data::node::{NodeColdData, NodeHotData, NodeWarmData};
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::persistent_vector::{
    RetainedVectorCapacityDenial, RetainedVectorMutationDenial, RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial, SignalConditionalRetentionDenial,
    SignalConditionalRetentionLedger, SignalConditionalRetentionReservation,
};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetainedNodeEditDenial {
    Lane(RetainedVectorStagingDenial),
    MissingHotPayload,
    EmptyNodeSelection,
    UnorderedNodeIndices,
    StorageChanged,
    Accounting(RetainedStoragePreparationDenial),
    Retention(SignalConditionalRetentionDenial),
    CapacityExhausted { maximum: Charge, required: Charge },
}

#[derive(Debug)]
pub(crate) enum RetainedNodeEditPreparation<R> {
    Ready(PreparedRetainedNodeEdit<R>),
    Rejected {
        output: R,
        denial: RetainedNodeEditDenial,
    },
}

#[derive(Debug)]
pub(crate) struct PreparedRetainedNodeEdit<R> {
    original: NodeEvaluationRoots,
    staged: NodeEvaluationRoots,
    output: R,
    charge: Charge,
    _preparation_custody: SignalConditionalRetentionReservation,
}

#[derive(Debug)]
struct NodeEvaluationRoots {
    hot: PersistentPagedVector<Option<NodeHotData>>,
    warm: PersistentPagedVector<NodeWarmData>,
    cold: PersistentPagedVector<Option<Box<NodeColdData>>>,
    custody: Option<Arc<SignalConditionalRetentionReservation>>,
}

impl<R> PreparedRetainedNodeEdit<R> {
    pub(crate) fn try_split_output<S, T, E>(
        self,
        split: impl FnOnce(R) -> Result<(S, T), E>,
    ) -> Result<(PreparedRetainedNodeEdit<S>, T), E> {
        let Self {
            original,
            staged,
            output,
            charge,
            _preparation_custody,
        } = self;
        let (output, peer) = split(output)?;
        Ok((
            PreparedRetainedNodeEdit {
                original,
                staged,
                output,
                charge,
                _preparation_custody,
            },
            peer,
        ))
    }

    /// This validates storage freshness only. The publication owner must
    /// separately validate node/basis authority and every other prepared owner
    /// before its first installation.
    pub(crate) fn storage_preconditions_hold(&self, arena: &NodeArena) -> bool {
        self.original
            .hot
            .matches_retained_edit_preconditions(&arena.hot)
            && self
                .original
                .warm
                .matches_retained_edit_preconditions(&arena.warm)
            && self
                .original
                .cold
                .matches_retained_edit_preconditions(&arena.cold)
    }

    pub(crate) fn install(self, arena: &mut NodeArena) -> RetainedNodeEditOutcome<R> {
        if !self.storage_preconditions_hold(arena) {
            return RetainedNodeEditOutcome::Rejected {
                output: self.output,
                denial: RetainedNodeEditDenial::StorageChanged,
            };
        }
        let NodeEvaluationRoots {
            hot,
            warm,
            cold,
            custody,
        } = self.staged;
        arena.hot = hot;
        arena.warm = warm;
        arena.cold = cold;
        arena.retained_node_custody = custody;
        RetainedNodeEditOutcome::Installed {
            output: self.output,
            charge: self.charge,
        }
    }
}

#[derive(Debug)]
pub(crate) enum RetainedNodeEditOutcome<R> {
    Installed {
        output: R,
        charge: Charge,
    },
    Rejected {
        output: R,
        denial: RetainedNodeEditDenial,
    },
}

/// Payloads correspond to the caller's strictly ordered node indices. Only
/// evaluation data enters this draft; definition and node authority stay with
/// the publication owner.
#[derive(Debug)]
pub(crate) struct RetainedNodePayload {
    pub(crate) hot: NodeHotData,
    pub(crate) warm: NodeWarmData,
    pub(crate) cold: Option<Box<NodeColdData>>,
}

impl NodeArena {
    /// The ceiling covers the three installed evaluation representations,
    /// including untouched backing. The ledger reserves selected payload clones
    /// and staged roots before their allocations. New payload allocations made
    /// by the callback and its returned output require caller reservations.
    pub(crate) fn prepare_retained_node_edits<R>(
        &self,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        indices: &[usize],
        maximum: Charge,
        work: &mut Preparation,
        edit: impl FnOnce(&mut [RetainedNodePayload]) -> R,
    ) -> Result<RetainedNodeEditPreparation<R>, RetainedNodeEditDenial> {
        self.validate_retained_node_selection(indices, work)?;
        work.reserve_visits(indices.len())
            .map_err(RetainedNodeEditDenial::Accounting)?;
        let draft_custody =
            reservation_admission::reserve_payload_draft(self, indices.len(), ledger, work)?;
        let mut payloads = indices
            .iter()
            .map(|&index| RetainedNodePayload {
                hot: self.hot[index]
                    .as_ref()
                    .expect("validated hot payload")
                    .clone(),
                warm: self.warm[index].clone(),
                cold: self.cold[index].clone(),
            })
            .collect::<Vec<_>>();
        let output = edit(&mut payloads);
        let mut staged = NodeEvaluationRoots {
            hot: self.hot.clone(),
            warm: self.warm.clone(),
            cold: self.cold.clone(),
            custody: self.retained_node_custody.clone(),
        };
        let prepared = (|| {
            let mut charge = Charge::ZERO;
            for (&index, payload) in indices.iter().zip(payloads) {
                let RetainedNodePayload { hot, warm, cold } = payload;
                let before = [
                    staged
                        .hot
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                    staged
                        .warm
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                    staged
                        .cold
                        .prepared_retained_charge()
                        .map_err(map_mutation_denial)?,
                ];
                let hot = staged
                    .hot
                    .prepare_retained_replacement(index, Some(hot), work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let warm = staged
                    .warm
                    .prepare_retained_replacement(index, warm, work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let cold = staged
                    .cold
                    .prepare_retained_replacement(index, cold, work)
                    .map_err(|(_, denial)| map_mutation_denial(denial))?;
                let required = publication_peak_charge(
                    before,
                    [
                        hot.required_charge(),
                        warm.required_charge(),
                        cold.required_charge(),
                    ],
                )
                .map_err(RetainedNodeEditDenial::Accounting)?;
                if required > maximum {
                    return Err(RetainedNodeEditDenial::CapacityExhausted { maximum, required });
                }
                let custody = reservation_admission::reserve_staged_roots(ledger, required)?;
                // All three destination borrows remain held through admission.
                // No selected lane allocates replacement storage before this
                // ceiling for every publication prefix has been checked.
                let hot_charge = hot
                    .publish(maximum)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let warm_charge = warm
                    .publish(maximum)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                let cold_charge = cold
                    .publish(maximum)
                    .map_err(|(_, d)| map_capacity_denial(d))?;
                staged.custody = Some(custody);
                charge = hot_charge
                    .checked_add(warm_charge)
                    .and_then(|n| n.checked_add(cold_charge))
                    .map_err(RetainedNodeEditDenial::Accounting)?;
            }
            Ok(charge)
        })();
        let charge = match prepared {
            Ok(charge) => charge,
            Err(denial) => return Ok(RetainedNodeEditPreparation::Rejected { output, denial }),
        };
        Ok(RetainedNodeEditPreparation::Ready(
            PreparedRetainedNodeEdit {
                original: NodeEvaluationRoots {
                    hot: self.hot.clone(),
                    warm: self.warm.clone(),
                    cold: self.cold.clone(),
                    custody: self.retained_node_custody.clone(),
                },
                staged,
                output,
                charge,
                _preparation_custody: draft_custody,
            },
        ))
    }

    fn validate_retained_node_selection(
        &self,
        indices: &[usize],
        work: &mut Preparation,
    ) -> Result<(), RetainedNodeEditDenial> {
        if indices.is_empty() {
            return Err(RetainedNodeEditDenial::EmptyNodeSelection);
        }
        // Cover order checks, lane validation, payload cloning, and draft installation
        // loops before any allocation or caller edit. Nested storage visits are
        // separately charged by the payload and persistent-container owners.
        for _ in 0..4 {
            work.reserve_visits(indices.len())
                .map_err(RetainedNodeEditDenial::Accounting)?;
        }
        if indices.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(RetainedNodeEditDenial::UnorderedNodeIndices);
        }
        // Validation, copy admission, and the actual clone each read every
        // selected lane. Admit their tree lookups before entering those loops.
        let lookups = [
            self.hot.lookup_steps(),
            self.warm.lookup_steps(),
            self.cold.lookup_steps(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, steps| sum.checked_add(steps))
        .and_then(|steps| steps.checked_mul(3))
        .and_then(|steps| steps.checked_mul(indices.len()))
        .ok_or(RetainedNodeEditDenial::Accounting(
            RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits: work.maximum_visits(),
            },
        ))?;
        work.reserve_visits(lookups)
            .map_err(RetainedNodeEditDenial::Accounting)?;
        for &index in indices {
            self.hot
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.warm
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.cold
                .validate_staged_edit(index)
                .map_err(RetainedNodeEditDenial::Lane)?;
            self.hot[index]
                .as_ref()
                .ok_or(RetainedNodeEditDenial::MissingHotPayload)?;
            work.reserve_visits(std::mem::size_of::<NodeHotData>() + 32)
                .map_err(RetainedNodeEditDenial::Accounting)?;
            let mut copies = crate::logic::evaluation::EvaluationWork::Conditional(work);
            self.warm[index]
                .admit_clone_work(&mut copies)
                .map_err(map_clone_work_denial)?;
            if let Some(cold) = &self.cold[index] {
                cold.admit_clone_work(&mut copies)
                    .map_err(map_clone_work_denial)?;
            }
        }
        Ok(())
    }
}

fn map_mutation_denial(denial: RetainedVectorMutationDenial) -> RetainedNodeEditDenial {
    match denial {
        RetainedVectorMutationDenial::Accounting(denial) => {
            RetainedNodeEditDenial::Accounting(denial)
        }
        denial => RetainedNodeEditDenial::Lane(RetainedVectorStagingDenial::Mutation(denial)),
    }
}

fn map_capacity_denial(denial: RetainedVectorCapacityDenial) -> RetainedNodeEditDenial {
    match denial {
        RetainedVectorCapacityDenial::Accounting(denial) => {
            RetainedNodeEditDenial::Accounting(denial)
        }
        RetainedVectorCapacityDenial::CapacityExhausted { maximum, required } => {
            RetainedNodeEditDenial::CapacityExhausted { maximum, required }
        }
    }
}

/// The installed lane order is hot, warm, cold. A later shrinking replacement
/// cannot fund an earlier growing one: admit every intermediate root tuple.
fn publication_peak_charge(
    before: [Charge; 3],
    after: [Charge; 3],
) -> Result<Charge, RetainedStoragePreparationDenial> {
    let mut current = before
        .into_iter()
        .try_fold(Charge::ZERO, Charge::checked_add)?;
    let mut peak = current;
    for (old, next) in before.into_iter().zip(after) {
        current = current.checked_sub(old)?.checked_add(next)?;
        peak = peak.max(current);
    }
    Ok(peak)
}

fn map_clone_work_denial(error: crate::data::error::SignalError) -> RetainedNodeEditDenial {
    match error {
        crate::data::error::SignalError::ConditionalEvaluationWorkExhausted { maximum_visits } => {
            RetainedNodeEditDenial::Accounting(RetainedStoragePreparationDenial::WorkExhausted {
                maximum_visits,
            })
        }
        _ => unreachable!("conditional payload clone admission returns only work exhaustion"),
    }
}
