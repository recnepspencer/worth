//! Atomic installation of cumulative retained node evaluation changes.
mod full_draft;
mod reservation_admission;
mod selected_epoch;
mod selection_admission;
use super::NodeArena;
use crate::data::node::{NodeColdData, NodeHotData, NodeWarmData};
use crate::data::persistent_paged_vector::PersistentPagedVector;
use crate::data::persistent_vector::{
    RetainedVectorCapacityDenial, RetainedVectorMutationDenial, RetainedVectorStagingDenial,
};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial, SignalConditionalRetentionDenial,
    SignalConditionalRetentionReservation,
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
            #[cfg(test)]
            charge: self.charge,
        }
    }
}

#[derive(Debug)]
pub(crate) enum RetainedNodeEditOutcome<R> {
    Installed {
        output: R,
        #[cfg(test)]
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

/// The selected role is fixed before callbacks and cannot be upgraded after
/// the graph begins preparing an epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectedNodeRole {
    ProducerFull,
    ConsumerOperational,
}

#[derive(Debug)]
pub(crate) struct OperationalNodePayload {
    pub(crate) hot: NodeHotData,
    pub(crate) warm: NodeWarmData,
}

#[derive(Debug)]
pub(crate) enum SelectedNodeDraft {
    ProducerFull(RetainedNodePayload),
    ConsumerOperational(OperationalNodePayload),
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
        #[cfg(test)]
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
