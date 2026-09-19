//! Infallible installation of one fully prepared retained output publication.

use super::snapshot_preparation::MaterializedEffectSnapshot;
use super::SignalGraph;
use crate::data::graph::runtime::graph::{
    PreparedRetainedDirectCauseStores, PreparedRetainedNodeEdit, RetainedNodeEditOutcome,
};
use crate::data::graph::PreparedRetainedPendingRevalidationIndex;
use crate::data::handle::NodeId;

#[derive(Debug)]
pub(super) struct PreparedRetainedOutputPublication {
    pub(super) roots: PreparedRetainedNodeEdit<super::super::EffectStateMutation>,
    pub(super) waiters: Option<PreparedRetainedPendingRevalidationIndex>,
    pub(super) stores: Option<PreparedRetainedDirectCauseStores>,
    pub(super) snapshot: Option<MaterializedEffectSnapshot>,
    pub(super) node: NodeId,
    pub(super) release: Option<crate::data::graph::storage::invalidation_causes::PendingCauseSetId>,
    pub(super) causality_changed: bool,
    pub(super) runtime_write: bool,
    pub(super) suppressed_downstream: u64,
}

impl PreparedRetainedOutputPublication {
    pub(super) fn publish(self, graph: &mut SignalGraph) {
        assert!(
            self.roots.storage_preconditions_hold(&graph.arena),
            "prepared output roots remain current"
        );
        graph
            .release_output_producer_causes(self.release)
            .expect("prevalidated producer release");
        let mutation = match self.roots.install(&mut graph.arena) {
            RetainedNodeEditOutcome::Installed { output, .. } => output,
            RetainedNodeEditOutcome::Rejected { .. } => {
                unreachable!("exclusive prepared output installation")
            }
        };
        if let Some(waiters) = self.waiters {
            waiters.install(graph);
        }
        graph.record_output_node_publication(
            self.node,
            mutation,
            self.causality_changed,
            self.runtime_write,
        );
        if let Some(snapshot) = self.snapshot {
            snapshot.publish(graph);
        }
        if let Some(stores) = self.stores {
            stores.publish(graph);
        }
    }
}
