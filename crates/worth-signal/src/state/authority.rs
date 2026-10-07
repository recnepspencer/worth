use serde::{Deserialize, Serialize};

use crate::data::graph::{DependencyEdgeStore, SubscriberEdgeStore};
use crate::data::node::CheckpointNodeImage;
use crate::data::proof::SnapshotBatchCommit;
use crate::data::telemetry::RuntimeTelemetry;
use crate::diagnostics::state::DiagnosticsState;
use crate::runtime_policy::InstalledSignalRuntimePolicy;

mod slot_wire;

#[derive(Debug, Clone, Serialize)]
pub struct SignalCheckpointSlot {
    pub node: Option<CheckpointNodeImage>,
    pub generation: u32,
    pub retired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCheckpointArena {
    pub slots: Vec<SignalCheckpointSlot>,
    pub free_list: Vec<u32>,
    pub active_nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCheckpointTopology {
    pub dependency_edges: DependencyEdgeStore,
    pub subscriber_edges: SubscriberEdgeStore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Narrow checkpoint-owned authority payload used to reconstruct operational
/// graph truth without carrying runtime observation baggage.
pub struct SignalCheckpointAuthority {
    pub(crate) arena: SignalCheckpointArena,
    pub(crate) topology: SignalCheckpointTopology,
    #[serde(
        default,
        serialize_with = "crate::data::graph::storage::invalidation_causes::serialize_canonical_cause_sets"
    )]
    pub(crate) cause_sets: crate::data::graph::storage::invalidation_causes::CanonicalCauseSetStore,
    pub(crate) diagnostics: DiagnosticsState,
    #[serde(default)]
    pub(crate) installed_policy: InstalledSignalRuntimePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Canonical checkpoint-carried authority image for reconstructive restore.
///
/// Supported restore paths must consume this image rather than treating the
/// entire snapshot bundle as the authority carrier.
pub struct SignalCheckpointImage {
    pub authority: SignalCheckpointAuthority,
    pub dependency_snapshot_batch: SnapshotBatchCommit,
    pub graph_telemetry: RuntimeTelemetry,
}
