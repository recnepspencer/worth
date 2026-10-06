//! Cumulative cause-store draft for one canonical graph publication epoch.
use super::{
    CanonicalCauseSetStore, NormalizedCauseSet, PendingCauseSetId, PreparedCauseSlot,
    PreparedRetainedCauseStorePublication,
};
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::invalidation::output_commit::ProducedAspectDelta;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth, RetainedStoragePreparation as Work,
};

pub(crate) enum PreparedEpochCauseStore {
    Ordinary(CanonicalCauseSetStore),
    Retained(PreparedRetainedCauseStorePublication),
}

/// Private draft edits in the same order as canonical producer settlement.
pub(crate) enum EpochCauseStoreEdit {
    Release(PendingCauseSetId),
    Transition(PreparedCauseSlot),
}

impl SignalGraph {
    pub(crate) fn epoch_cause_store_fork_growth_bound(&self) -> Result<u64, SignalError> {
        Ok(self.cause_sets.epoch_fork_growth_bound()?.bytes())
    }

    pub(crate) fn prepare_epoch_cause_store(
        &mut self,
        edits: Vec<EpochCauseStoreEdit>,
        finals: Vec<(PendingCauseSetId, NormalizedCauseSet)>,
        commits: Vec<ProducedAspectDelta>,
        work: &mut Work,
        preparation: Option<&mut SignalPreparationBudget>,
    ) -> Result<PreparedEpochCauseStore, SignalError> {
        if let Some(budget) = preparation {
            budget.claim(self.epoch_cause_store_fork_growth_bound()?)?;
        }
        if let Some(ledger) = self.arena.retained_node_ledger.clone() {
            let maximum = usize::try_from(
                self.installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_retained_bytes,
            )
            .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let maximum = Charge::capacity::<u8>(maximum)
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            let mut draft = self.begin_retained_cause_store_publication(&ledger, maximum)?;
            for edit in edits {
                match edit {
                    EpochCauseStoreEdit::Release(release) => {
                        draft.release_prepared_producer(release, work)?;
                    }
                    EpochCauseStoreEdit::Transition(slot) => {
                        draft.apply_epoch_virtual_slot(slot, work)?;
                    }
                }
            }
            for (handle, causes) in finals {
                draft.finish_epoch_virtual_slot(handle, causes, work)?;
            }
            for commit in commits {
                draft.publish_output_commit(commit, work)?;
            }
            Ok(PreparedEpochCauseStore::Retained(draft.finish()?))
        } else {
            self.cause_sets
                .prepare_fork_growth(work)
                .map_err(|_| SignalError::EvaluationStorageUnavailable)?;
            let mut draft = self.cause_sets.fork_persistent();
            for edit in edits {
                match edit {
                    EpochCauseStoreEdit::Release(release) => {
                        if release != PendingCauseSetId::EMPTY {
                            draft.release_epoch_accounted(release, work)?;
                        }
                    }
                    EpochCauseStoreEdit::Transition(slot) => {
                        draft.apply_epoch_virtual_slot(slot, work)?;
                    }
                }
            }
            for (handle, causes) in finals {
                draft.finish_epoch_virtual_slot(handle, causes, work)?;
            }
            for commit in commits {
                draft.publish_epoch_output_commit_accounted(commit, work)?;
            }
            Ok(PreparedEpochCauseStore::Ordinary(draft))
        }
    }
}

impl PreparedEpochCauseStore {
    pub(crate) fn publish(self, graph: &mut SignalGraph) {
        match self {
            Self::Ordinary(store) => {
                graph.cause_sets = store;
            }
            Self::Retained(store) => {
                graph.install_retained_cause_store_publication(store);
            }
        }
    }
}
