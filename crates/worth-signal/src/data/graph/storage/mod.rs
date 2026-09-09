mod diagnostic_scan;
mod entries;
pub(in crate::data::graph) use entries::NodeEvaluationMutation;
pub(crate) use entries::PreparedInvalidationCache;
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
pub(crate) use segmented::{DependencyEdgeStore, SubscriberEdgeStore};
pub(crate) use slot::Slot;
