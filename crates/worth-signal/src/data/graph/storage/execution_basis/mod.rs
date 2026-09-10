//! Selected graph storage. Branch admission remains the owner's responsibility.

mod capture_preparation;
mod definitions;
mod evaluation_seed;
mod partition_admission;
mod retained_charge;
use crate::data::retained_storage::RetainedStorageCharge;
#[cfg(test)]
use crate::data::retained_storage::RetainedStoragePreparation;
pub(crate) use retained_charge::SignalExecutionBasisChargeDenial;

#[cfg(test)]
use super::evaluation_partition::SignalEvaluationPartition;
#[cfg(test)]
use crate::data::graph::SignalGraph;
pub(in crate::data::graph) use definitions::SignalExecutionDefinitions;
pub(in crate::data::graph) use evaluation_seed::SignalEvaluationStorage;

/// Immutable captured roots, retained by an exact owner observation. Creating
/// a mutable slot never reads the current graph or modifies this backing.
/// Capture must occur under owner admission and its resource reservation.
#[derive(Debug)]
pub(crate) struct SignalExecutionBasis {
    definitions: SignalExecutionDefinitions,
    evaluation: SignalEvaluationStorage,
    retained_charge: RetainedStorageCharge,
}

impl SignalExecutionBasis {
    #[cfg(test)]
    pub(crate) fn retained_node_observation(
        &self,
        node: crate::data::handle::NodeId,
        aspect: crate::data::aspect::Aspect,
    ) -> Option<(crate::data::node::NodeState, u64, bool)> {
        self.definitions
            .contains_node(node)
            .then(|| self.evaluation.retained_node_observation(node, aspect))
            .flatten()
    }

    /// Explicit storage fixture, without owner admission or resource custody.
    #[cfg(test)]
    pub(crate) fn capture(
        graph: &mut SignalGraph,
        work: &mut RetainedStoragePreparation,
    ) -> Result<Self, SignalExecutionBasisChargeDenial> {
        Ok(Self::prepare_capture(graph, work)?.capture_for_test())
    }

    #[cfg(test)]
    pub(crate) fn new_evaluation_partition(&self) -> SignalEvaluationPartition {
        let maximum = self
            .definitions
            .installed_policy()
            .conditional_evaluation_budget()
            .maximum_attempt_visits;
        self.try_new_evaluation_partition(&mut RetainedStoragePreparation::new(maximum))
            .expect("storage fixture admits initial partition against its installed budget")
    }
}
