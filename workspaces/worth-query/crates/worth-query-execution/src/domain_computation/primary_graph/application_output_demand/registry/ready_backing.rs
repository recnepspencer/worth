//! One prepared Ready cell, retained through every reader of its completion.
//!
//! The cell also keeps the mode a required wave executes the output's producer
//! in again. An executed output has the mode that executed it. A restored
//! output was executed by no demand of this runtime: it takes the mode of the
//! first advance that reaches it, its own demand's or that of a dependent
//! whose chain consumed it.

use std::ops::Deref;
use std::sync::Arc;
use std::sync::OnceLock;

use super::WorthQueryCompletedOutputDemand;

type ProducerMode =
    crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority;

struct ReadyCell {
    completion: OnceLock<WorthQueryCompletedOutputDemand>,
    producer_mode: OnceLock<ProducerMode>,
}

/// Prepared before an output effect. It cannot be read as a Ready completion.
pub(in crate::domain_computation::primary_graph) struct PreparedReadyBacking {
    cell: Arc<ReadyCell>,
}

/// The completed cell pins the original authority.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct ReadyCompletion(Arc<ReadyCell>);

impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn has_required_row_for_test(
        &self,
        root: worth_relational::facade::identity::EntityId,
    ) -> bool {
        let state = self
            .output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        state.records.iter().any(|(key, record)| {
            key.source.root_entity_for_test() == root && !record.settlements.is_empty()
        })
    }
}

impl PreparedReadyBacking {
    pub(super) fn new() -> Self {
        Self {
            cell: Arc::new(ReadyCell {
                completion: OnceLock::new(),
                producer_mode: OnceLock::new(),
            }),
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn for_test() -> Self {
        Self::new()
    }

    /// Bind the mode selected for the actual producer execution before it
    /// publishes either a performed result or a Stable equality result.
    pub(in crate::domain_computation::primary_graph) fn bind_execution_mode(
        self,
        mode: ProducerMode,
    ) -> Self {
        assert!(self.cell.producer_mode.set(mode).is_ok());
        self
    }

    pub(in crate::domain_computation::primary_graph) fn complete(
        self,
        completion: WorthQueryCompletedOutputDemand,
    ) -> ReadyCompletion {
        assert!(self.cell.completion.set(completion).is_ok());
        ReadyCompletion(self.cell)
    }
}

impl Deref for ReadyCompletion {
    type Target = WorthQueryCompletedOutputDemand;

    fn deref(&self) -> &Self::Target {
        self.0
            .completion
            .get()
            .expect("only a filled Ready cell is observable")
    }
}

impl ReadyCompletion {
    pub(in crate::domain_computation::primary_graph) fn same_cell(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// The mode a required wave executes this output's producer in again.
    /// A restored output has none until an advance reaches it.
    pub(in crate::domain_computation::primary_graph) fn producer_mode(
        &self,
    ) -> Option<&ProducerMode> {
        self.0.producer_mode.get()
    }

    /// An advance in `mode` reaches this output: its own demand's, or a
    /// dependent's whose chain consumed it. A restored output keeps the first
    /// such mode; an executed one keeps its own.
    pub(in crate::domain_computation::primary_graph) fn advanced_in(&self, mode: &ProducerMode) {
        if self.0.producer_mode.get().is_none() {
            drop(self.0.producer_mode.set(mode.clone()));
        }
    }
}

#[cfg(test)]
impl ReadyCompletion {
    pub(in crate::domain_computation::primary_graph) fn for_test(
        completion: WorthQueryCompletedOutputDemand,
    ) -> Self {
        PreparedReadyBacking::new().complete(completion)
    }
}

#[cfg(test)]
impl PreparedReadyBacking {
    pub(in crate::domain_computation::primary_graph) fn for_registry_test() -> Self {
        Self::new()
    }
}
