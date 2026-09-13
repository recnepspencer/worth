//! Prepared cause-store writes; node caches and lifecycle are a separate output.
use super::super::PreparedDirectCounterDeltas;
use super::{PreparedCauseNodeReplacement, PreparedDirectCauseAdmission};
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::{NormalizedCauseSet, PreparedCauseSlot};
use crate::data::graph::{PendingRevalidationPreparationDenial, SignalGraph};
use crate::data::handle::NodeId;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::retained_storage::RetainedStoragePreparation;
use crate::data::retained_storage::{RetainedStorageCharge, SignalConditionalRetentionLedger};
use std::sync::Arc;

#[derive(Debug)]
pub(super) struct PreparedCauseStoreReplacement {
    pub(super) consumer: NodeId,
    pub(super) causes: NormalizedCauseSet,
    pub(super) slot: PreparedCauseSlot,
}

#[derive(Debug)]
pub(crate) struct PreparedDirectCauseStores {
    pub(super) producer: NodeId,
    pub(super) commit: Option<ProducedAspectDelta>,
    pub(super) replacements: Vec<PreparedCauseStoreReplacement>,
    counters: PreparedDirectCounterDeltas,
}

#[derive(Debug)]
pub(crate) struct PreparedRetainedDirectCauseStores {
    store: crate::data::graph::storage::invalidation_causes::PreparedRetainedCauseStorePublication,
    counters: PreparedDirectCounterDeltas,
}

impl PreparedDirectCauseStores {
    pub(super) fn split_node_changes(
        admission: PreparedDirectCauseAdmission,
        slots: Vec<PreparedCauseSlot>,
        work: &mut RetainedStoragePreparation,
    ) -> Result<(Self, Vec<PreparedCauseNodeReplacement>), SignalError> {
        assert_eq!(
            admission.replacements.len(),
            slots.len(),
            "every cause replacement has a prepared slot"
        );
        let count = slots.len();
        let visits = count
            .checked_mul(
                std::mem::size_of::<PreparedCauseStoreReplacement>()
                    + std::mem::size_of::<PreparedCauseNodeReplacement>()
                    + 2,
            )
            .filter(|bytes| *bytes <= isize::MAX as usize);
        crate::data::graph::waiter_preparation_work::reserve(work, visits)
            .map_err(PendingRevalidationPreparationDenial::into_signal_error)?;
        let mut stores = Vec::with_capacity(count);
        let mut nodes = Vec::with_capacity(count);
        for (replacement, slot) in admission.replacements.into_iter().zip(slots) {
            nodes.push(PreparedCauseNodeReplacement {
                consumer: replacement.consumer,
                cause_set: slot.handle(),
                cache: replacement.cache,
            });
            stores.push(PreparedCauseStoreReplacement {
                consumer: replacement.consumer,
                causes: replacement.causes,
                slot,
            });
        }
        // Admission walks normalized subscribers in NodeId order; filtering
        // direct-basis owners preserves that order. Publication merges this
        // carried stream with the ordered waiter projections without sorting.
        Ok((
            Self {
                producer: admission.producer,
                commit: admission.commit,
                replacements: stores,
                counters: admission.counter_deltas,
            },
            nodes,
        ))
    }

    pub(crate) fn publish(self, graph: &mut SignalGraph) -> Result<(), SignalError> {
        let commit = self.commit.clone();
        for replacement in self.replacements {
            graph.publish_prepared_cause_storage(
                replacement.consumer,
                replacement.causes,
                self.commit.as_ref(),
                replacement.slot,
            )?;
        }
        if let Some(commit) = commit {
            graph.cause_sets.publish_output_commit(commit);
        }
        self.counters.publish(graph);
        Ok(())
    }

    pub(crate) fn prepare_retained(
        self,
        graph: &SignalGraph,
        ledger: &Arc<SignalConditionalRetentionLedger>,
        maximum: RetainedStorageCharge,
        work: &mut RetainedStoragePreparation,
    ) -> Result<PreparedRetainedDirectCauseStores, SignalError> {
        let Self {
            producer: _,
            commit,
            replacements,
            counters,
        } = self;
        let mut draft = graph.begin_retained_cause_store_publication(ledger, maximum)?;
        for replacement in replacements {
            draft.publish_slot(replacement.slot, replacement.causes, work)?;
        }
        if let Some(commit) = commit {
            draft.publish_output_commit(commit, work)?;
        }
        Ok(PreparedRetainedDirectCauseStores {
            store: draft.finish()?,
            counters,
        })
    }
}

impl PreparedRetainedDirectCauseStores {
    pub(crate) fn publish(self, graph: &mut SignalGraph) {
        graph.install_retained_cause_store_publication(self.store);
        self.counters.publish(graph);
    }
}
