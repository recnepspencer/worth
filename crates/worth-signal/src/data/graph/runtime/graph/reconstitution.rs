use crate::data::error::SignalError;
use crate::state::SignalCheckpointImage;

use super::SignalGraph;

/// Fresh Signal runtime state reconstructed from graph-owned checkpoint
/// authority. The checkpoint image and its derived indexes never leave the
/// Signal owner.
pub struct SignalGraphReconstitution {
    graph: SignalGraph,
    report: SignalGraphReconstitutionReport,
}

impl SignalGraphReconstitution {
    pub fn into_parts(self) -> (SignalGraph, SignalGraphReconstitutionReport) {
        (self.graph, self.report)
    }

    pub const fn report(&self) -> SignalGraphReconstitutionReport {
        self.report
    }
}

/// Runtime-local observation of one completed checkpoint reconstruction.
///
/// This report describes reconstructive work. It is not execution authority
/// and cannot admit invalidation work in either the old or restored graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SignalGraphReconstitutionReport {
    previous_graph_instance_id: u64,
    restored_graph_instance_id: u64,
    reconstructed_node_count: usize,
    checkpoint_reconstruction_count: u64,
}

impl SignalGraphReconstitutionReport {
    pub const fn previous_graph_instance_id(self) -> u64 {
        self.previous_graph_instance_id
    }

    pub const fn restored_graph_instance_id(self) -> u64 {
        self.restored_graph_instance_id
    }

    pub const fn reconstructed_node_count(self) -> usize {
        self.reconstructed_node_count
    }

    pub const fn checkpoint_reconstruction_count(self) -> u64 {
        self.checkpoint_reconstruction_count
    }
}

impl SignalGraph {
    /// Reconstructs a fresh graph from canonical checkpoint authority.
    ///
    /// Dependency, subscriber, reverse-subscription, partition, cause, and
    /// structural-pending indexes are rebuilt by the supported restore path.
    /// Ready work and performed counters are intentionally not checkpoint
    /// authority.
    pub fn reconstitute_for_runtime_rebind(
        &self,
    ) -> Result<SignalGraphReconstitution, SignalError> {
        let previous_graph_instance_id = self.installed_graph_capability().graph_instance_id();
        let checkpoint = SignalCheckpointImage {
            authority: self.capture_checkpoint_authority(),
            dependency_snapshot_batch: self.capture_checkpoint_dependency_snapshot_batch(),
            graph_telemetry: *self.telemetry(),
        };
        let mut graph = Self::restore_from_checkpoint_image(&checkpoint)?;
        graph.telemetry_clock = self.telemetry_clock.clone();
        let report = SignalGraphReconstitutionReport {
            previous_graph_instance_id,
            restored_graph_instance_id: graph.installed_graph_capability().graph_instance_id(),
            reconstructed_node_count: graph.live_node_ids().len(),
            checkpoint_reconstruction_count: graph.checkpoint_reconstruction_count(),
        };
        Ok(SignalGraphReconstitution { graph, report })
    }
}
