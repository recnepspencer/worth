mod diagnostic_scan;
mod entries;
pub(crate) use entries::PreparedInvalidationCache;
pub(in crate::data::graph) use entries::{ConsumerNodeMutation, NodeEvaluationMutation};
pub(crate) mod evaluation_partition;
pub(crate) mod execution_basis;
mod handles;
pub(crate) mod invalidation_causes;
mod segmented;
mod slot;

pub(crate) use diagnostic_scan::GraphDiagnosticNode;
pub(crate) use handles::{DependencySetId, SubscriberSetId};
#[cfg(test)]
pub(crate) use segmented::checked_segment_component_for_test;
pub(crate) use segmented::{
    DependencyEdgeStore, PreparedSegmentBatchInsertion, SubscriberEdgeStore,
};
pub(crate) use slot::Slot;
