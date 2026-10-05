use serde::{Deserialize, Serialize};

use worth_signal::diagnostics::ReplayEvent;
use worth_signal::facade::diagnostics::GraphSummary;
use worth_signal::facade::history::{
    LineageEvent as NativeLineageEvent, ReplayView, RuntimeSnapshot,
};

use crate::expression::model::SignalValue;
use crate::runtime::compute_callbacks::CapturedHostCapabilityRead;
mod callback_dependencies;
pub(crate) use callback_dependencies::{
    public_callback_dependency_patch_summary, public_callback_read_ids,
};
mod execution_report;
pub use execution_report::WebExecutionReportSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AspectVersionSummary {
    pub aspect: u8,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub touched_nodes: u32,
    pub nodes_evaluated: u32,
    pub nodes_recomputed: u32,
    pub nodes_suppressed: u32,
    pub plans_built: u32,
    pub stages_executed: u32,
    pub total_nanos: String,
    pub evaluation_nanos: String,
    pub commit_nanos: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhySummary {
    pub id: String,
    pub node: String,
    pub api_family: Option<String>,
    pub recipe_family: Option<String>,
    pub state: String,
    pub upstream: Vec<String>,
    pub changed_regions: Vec<String>,
    pub propagation_suppressed: bool,
    pub output_change: Option<String>,
    pub output_identity: Option<String>,
    pub callback: Option<CallbackWhySummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackWhySummary {
    pub purity_posture: String,
    pub current_reads: Vec<String>,
    pub host_capability_reads: Vec<CapturedHostCapabilityRead>,
    pub registered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_generation: Option<u64>,
    pub last_runtime_read_breadth: u64,
    pub last_dependency_patch: Option<CallbackDependencyPatchSummary>,
    pub last_failure: Option<CallbackFailureSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackRuntimeNodeSummary {
    pub id: String,
    pub node: String,
    pub api_family: Option<String>,
    pub recipe_family: Option<String>,
    pub purity_posture: String,
    pub current_reads: Vec<String>,
    pub host_capability_reads: Vec<CapturedHostCapabilityRead>,
    pub registered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_slot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_generation: Option<u64>,
    pub last_runtime_read_breadth: u64,
    pub last_dependency_patch: Option<CallbackDependencyPatchSummary>,
    pub last_failure: Option<CallbackFailureSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackDependencyPatchSummary {
    pub previous_reads: Vec<String>,
    pub current_reads: Vec<String>,
    pub added_count: u64,
    pub removed_count: u64,
    pub retained_count: u64,
    pub runtime_read_breadth: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallbackFailureSummary {
    pub class: String,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSummary {
    pub active_node_count: u32,
    pub clean_node_count: u32,
    pub maybe_stale_node_count: u32,
    pub dirty_node_count: u32,
    pub dependency_edge_count: u32,
    pub subscriber_edge_count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayFrameSummary {
    pub cursor: u64,
    pub kind: String,
    pub branch_id: u64,
    pub snapshot_id: Option<u64>,
    pub node: Option<String>,
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback: Option<CallbackRuntimeNodeSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaySummary {
    pub frames: Vec<ReplayFrameSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineageSummary {
    pub events: Vec<LineageEventSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineageEventSummary {
    pub sequence: u64,
    pub label: String,
    pub emitted_on_branch_id: u64,
    pub node: Option<String>,
    pub subject_artifact_id: Option<u64>,
    pub parent_artifact_id: Option<u64>,
    pub snapshot_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback: Option<CallbackRuntimeNodeSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionHistorySurfaceSummary {
    pub history: worth_signal::facade::diagnostics::ExecutionHistorySummary,
    pub callback_nodes: Vec<CallbackRuntimeNodeSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowSurfaceSummary {
    pub flow: worth_signal::diagnostics::FlowSummary,
    pub callback_nodes: Vec<CallbackRuntimeNodeSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservationSurfaceSummary {
    pub observation: worth_signal::facade::runtime::ObservationBoundarySummary,
    pub callback_nodes: Vec<CallbackRuntimeNodeSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebPerformanceSummary {
    pub active_handle_count: u64,
    pub active_callback_count: u64,
    pub active_compute_callback_count: u64,
    pub active_compute_collector_count: u64,
    pub matched_watcher_breadth: u64,
    pub delivered_observation_count: u64,
    pub rollback_suppressed_delivery_count: u64,
    pub serial_executor_usage_count: u64,
    /// The latest report returned by the execution authority, when one ran.
    pub execution_report: Option<WebExecutionReportSummary>,
    pub output_serialization_count: u64,
    pub output_serialization_breadth: u64,
    pub js_callback_invocation_count: u64,
    pub js_callback_failure_count: u64,
    pub observation_callback_registration_count: u64,
    pub observation_callback_disposal_count: u64,
    pub observation_callback_generation_mismatch_denial_count: u64,
    pub observation_callback_allocation_count: u64,
    pub observation_callback_reuse_count: u64,
    pub compute_callback_registration_count: u64,
    pub compute_callback_disposal_count: u64,
    pub compute_callback_invocation_count: u64,
    pub compute_callback_failure_count: u64,
    pub compute_callback_generation_mismatch_denial_count: u64,
    pub compute_callback_self_read_denial_count: u64,
    pub compute_callback_dynamic_cycle_denial_count: u64,
    pub compute_callback_promise_return_denial_count: u64,
    pub compute_callback_invalid_return_denial_count: u64,
    pub compute_callback_collector_installation_count: u64,
    pub compute_callback_capture_count: u64,
    pub compute_callback_captured_read_count: u64,
    pub compute_callback_return_serialization_breadth: u64,
    pub compute_callback_allocation_count: u64,
    pub compute_callback_reuse_count: u64,
    pub compute_callback_dependency_patch_count: u64,
    pub compute_callback_dependency_patch_added_count: u64,
    pub compute_callback_dependency_patch_removed_count: u64,
    pub compute_callback_dependency_patch_retained_count: u64,
    pub compute_callback_runtime_read_breadth: u64,
    pub compute_callback_constant_no_signal_read_classification_count: u64,
    pub compute_callback_signal_tracked_classification_count: u64,
    pub compute_callback_missing_unavailability_count: u64,
    pub compatibility_read_count: u64,
    pub compatibility_read_breadth: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAsyncLifecycleCertification {
    pub node_id: String,
    pub payload_contract_id: u64,
    pub payload_byte_len: u64,
    pub request_admitted: bool,
    pub completion_committed: bool,
    pub resource_runtime_digest: String,
    pub resource_replay_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshotEnvelope {
    pub snapshot: RuntimeSnapshot,
    pub state: RuntimeStoreSnapshot,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStoreSnapshot {
    pub sources: Vec<StoredSourceSnapshot>,
    pub recipes: Vec<StoredRecipeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSourceSnapshot {
    pub id: String,
    pub value: SignalValue,
    pub version: u64,
    #[serde(default)]
    pub produces_aspects: Option<Vec<u8>>,
    #[serde(default)]
    pub aspect_versions: Vec<AspectVersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecipeSnapshot {
    pub id: String,
    pub value: SignalValue,
    pub version: u64,
    #[serde(default)]
    pub produces_aspects: Option<Vec<u8>>,
    #[serde(default)]
    pub aspect_versions: Vec<AspectVersionSummary>,
    pub initialized: bool,
    pub output_identity: Option<String>,
    #[serde(default)]
    pub callback: Option<StoredCallbackRecipeSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredCallbackRecipeSnapshot {
    pub token_slot: u64,
    pub token_generation: u64,
    pub reads: Vec<String>,
    #[serde(default)]
    pub host_capability_reads: Vec<CapturedHostCapabilityRead>,
}

impl From<GraphSummary> for HealthSummary {
    fn from(value: GraphSummary) -> Self {
        Self {
            active_node_count: value.active_node_count,
            clean_node_count: value.clean_node_count,
            maybe_stale_node_count: value.maybe_stale_node_count,
            dirty_node_count: value.dirty_node_count,
            dependency_edge_count: value.dependency_edge_count,
            subscriber_edge_count: value.subscriber_edge_count,
        }
    }
}

impl From<ReplayView> for ReplaySummary {
    fn from(value: ReplayView) -> Self {
        Self {
            frames: value
                .frames
                .into_iter()
                .map(ReplayFrameSummary::from)
                .collect(),
        }
    }
}

impl From<ReplayEvent> for ReplayFrameSummary {
    fn from(value: ReplayEvent) -> Self {
        Self {
            cursor: value.cursor.0,
            kind: format!("{:?}", value.kind),
            branch_id: value.branch_id.0,
            snapshot_id: value.snapshot_id.map(|id| id.0),
            node: value.node.map(|node| node.to_string()),
            detail: value
                .detail
                .and_then(|detail| detail.as_message().map(str::to_owned))
                .or_else(|| {
                    value
                        .execution_record_id
                        .map(|id| format!("executionRecord:{id}"))
                }),
            callback: None,
        }
    }
}

impl From<Vec<NativeLineageEvent>> for LineageSummary {
    fn from(value: Vec<NativeLineageEvent>) -> Self {
        Self {
            events: value
                .into_iter()
                .map(|record| LineageEventSummary {
                    sequence: record.sequence,
                    label: record.label().to_owned(),
                    emitted_on_branch_id: record.emitted_on_branch_id().0,
                    node: record.node().map(|node| node.to_string()),
                    subject_artifact_id: record.subject_artifact_id().map(|id| id.0),
                    parent_artifact_id: record.parent_artifact_id().map(|id| id.0),
                    snapshot_id: record.snapshot_id().map(|id| id.0),
                    callback: None,
                })
                .collect(),
        }
    }
}
