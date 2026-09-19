//! Coupled derived storage; admission and retention belong to the Signal owner.

#[cfg(test)]
use super::execution_basis::SignalExecutionBasis;
use super::execution_basis::{SignalEvaluationStorage, SignalExecutionDefinitions};
use crate::data::graph::signal_graph::{SignalGraph, TraversalResources};

mod activation;
mod admission;
#[cfg(test)]
mod admission_tests;
mod conditional_execution;
mod draft;
#[cfg(test)]
mod mutable_storage_tests;
mod observation;
mod unwind;
#[cfg(test)]
pub(in crate::data::graph) use draft::ConditionalEvaluationDraft;
pub(crate) use draft::SignalRejectedConditionalEvaluation;
pub(crate) use unwind::{SignalPartitionConditionalUnwind, SignalPartitionConditionalUnwindReason};

pub(crate) use conditional_execution::{
    SignalPartitionConditionalCompletion, SignalPartitionConditionalDenial,
};
use observation::EvaluationObservationStorage;

/// Mutable slot storage, never authority to select a source or definition.
/// The owner must initialize source-dependent validity before first execution.
pub(crate) struct SignalEvaluationPartition {
    definitions: SignalExecutionDefinitions,
    evaluation: SignalEvaluationStorage,
    observation: EvaluationObservationStorage,
    traversal: TraversalResources,
    pending_unwind: Option<SignalPartitionConditionalUnwind>,
    // Slot admission ends with the canonical partition; payload custody may escape.
    _slot_custody: Option<crate::data::retained_storage::SignalConditionalRetentionReservation>,
}

impl SignalEvaluationPartition {
    /// Storage-kernel convenience; owner admission retains a basis once and
    /// subsequently creates slots from that immutable captured backing.
    #[cfg(test)]
    pub(crate) fn retain_basis_storage(graph: &mut SignalGraph) -> Self {
        SignalExecutionBasis::capture(
            graph,
            &mut crate::data::retained_storage::RetainedStoragePreparation::new(100_000),
        )
        .expect("storage-kernel fixture admits bounded immutable seed preparation")
        .new_evaluation_partition()
    }

    #[cfg(test)]
    pub(in crate::data::graph) fn from_retained(
        definitions: SignalExecutionDefinitions,
        evaluation: SignalEvaluationStorage,
    ) -> Self {
        let observation = EvaluationObservationStorage::new(definitions.installed_policy(), None);
        Self {
            definitions,
            evaluation,
            observation,
            traversal: TraversalResources::default(),
            pending_unwind: None,
            _slot_custody: None,
        }
    }

    fn exchange(&mut self, graph: &mut SignalGraph) {
        self.definitions.exchange(graph);
        self.evaluation.exchange(graph);
        self.observation.exchange(graph);
        std::mem::swap(&mut self.traversal, &mut graph.traversal);
        // Readiness epochs stay monotonic on the owning graph across activation.
    }

    pub(crate) fn has_pending_conditional_unwind(&self) -> bool {
        self.pending_unwind.is_some()
    }

    pub(crate) fn has_retention_ledger(&self) -> bool {
        self.evaluation.has_retention_ledger()
    }

    pub(crate) fn prepare_persistent_fork_readiness(
        &mut self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<(), crate::data::error::SignalError> {
        self.evaluation
            .prepare_persistent_fork_readiness(work)
            .map_err(crate::data::graph::runtime::graph::map_node_edit_accounting)
    }

    pub(in crate::data::graph) fn try_fork_evaluation_storage(
        &mut self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<SignalEvaluationStorage, crate::data::error::SignalError> {
        self.evaluation.try_fork_persistent(work)
    }
}
