use std::collections::BTreeMap;

use serde_json::json;

use super::super::execution::HarnessExecution;
use super::super::{
    policy::terminal_report_export as policy_terminal_report_export,
    source::terminal_report_export as source_terminal_report_export,
    speculation::terminal_report_export as speculation_terminal_report_export,
    stream::terminal_report_export as stream_terminal_report_export,
    structural::terminal_report_export as structural_terminal_report_export,
    writeback::terminal_report_export as writeback_terminal_report_export,
};
use crate::harness::adapter::terminal_report_export::route_record_json;

pub(in crate::harness::adapter::adapter_impl) fn execution_summary_json(
    execution: &HarnessExecution,
) -> serde_json::Value {
    match execution {
        HarnessExecution::Route {
            result,
            continuity_summary,
        } => json!({
            "route_identity": result.result_summary().route_identity().as_str(),
            "invalidation_identity": result.result_summary().invalidation_identity().as_str(),
            "subscription_slice_identity": result.result_summary().subscription_slice_identity().as_str(),
            "snapshot_identity": result.result_summary().snapshot_identity().as_str(),
            "subscription_slice_count": result.result_summary().subscription_slice_count(),
            "delivered_target_count": result.result_summary().delivered_target_count(),
            "continuity_identity": continuity_summary
                .as_ref()
                .map(|(artifact, _)| artifact.continuity_identity().as_str()),
            "continuity_resolution_digest": continuity_summary
                .as_ref()
                .map(|(artifact, _)| artifact.continuity_resolution_digest()),
        }),
        HarnessExecution::Historical {
            artifact,
            record,
            explanation,
        } => json!({
            "historical_artifact_identity": artifact.artifact_identity().as_str(),
            "historical_record_identity": record.record_identity().as_str(),
            "declaration_identity": record.declaration().declaration_identity().as_str(),
            "snapshot_identity": explanation.snapshot_identity().as_str(),
            "branch_identity": explanation.branch_identity().as_str(),
            "commit_identity": explanation
                .commit_identity()
                .map(|identity| identity.as_str()),
            "materialization_path": format!("{:?}", explanation.materialization_path()),
            "selector_width": record.counters().selector_width(),
            "branch_width": record.counters().branch_width(),
        }),
        HarnessExecution::Stream(execution) => {
            stream_terminal_report_export::summary_json(execution)
        }
        HarnessExecution::Source(execution) => {
            source_terminal_report_export::summary_json(execution)
        }
        HarnessExecution::Policy(execution) => {
            policy_terminal_report_export::execution_summary_json(execution)
        }
        HarnessExecution::Speculation(execution) => {
            speculation_terminal_report_export::summary_json(execution)
        }
        HarnessExecution::Structural(execution) => {
            structural_terminal_report_export::summary_json(execution)
        }
        HarnessExecution::Writeback(execution) => {
            writeback_terminal_report_export::summary_json(execution)
        }
    }
}

pub(in crate::harness::adapter::adapter_impl) fn execution_extensions_json(
    execution: &HarnessExecution,
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> BTreeMap<String, serde_json::Value> {
    match execution {
        HarnessExecution::Route {
            result,
            continuity_summary,
        } => route_execution_extensions_json(result, continuity_summary, runtime_bridge),
        HarnessExecution::Historical {
            artifact,
            record,
            explanation,
        } => BTreeMap::from([(
            "bridge_historical_evaluation_record".to_string(),
            json!({
                "artifact_identity": artifact.artifact_identity().as_str(),
                "artifact_digest": artifact.digest(),
                "record_identity": record.record_identity().as_str(),
                "decision_log_identity": record.decision_log().decision_log_identity().as_str(),
                "declaration_identity": record.declaration().declaration_identity().as_str(),
                "snapshot_identity": explanation.snapshot_identity().as_str(),
                "branch_identity": explanation.branch_identity().as_str(),
                "commit_identity": explanation
                    .commit_identity()
                    .map(|identity| identity.as_str()),
                "materialization_path": format!("{:?}", explanation.materialization_path()),
                "counters": {
                    "truth_view_selector_count": record.counters().truth_view_selector_count(),
                    "historical_truth_view_count": record.counters().historical_truth_view_count(),
                    "branch_truth_view_count": record.counters().branch_truth_view_count(),
                    "planned_truth_view_packet_count": record.counters().planned_truth_view_packet_count(),
                    "resolved_truth_view_policy_count": record.counters().resolved_truth_view_policy_count(),
                    "materialized_truth_view_count": record.counters().materialized_truth_view_count(),
                    "truth_view_unavailable_count": record.counters().truth_view_unavailable_count(),
                    "truth_view_branch_mismatch_count": record.counters().truth_view_branch_mismatch_count(),
                    "truth_view_snapshot_mismatch_count": record.counters().truth_view_snapshot_mismatch_count(),
                    "historical_replay_mismatch_count": record.counters().historical_replay_mismatch_count(),
                    "branch_local_evaluation_count": record.counters().branch_local_evaluation_count(),
                    "truth_view_decision_log_count": record.counters().truth_view_decision_log_count(),
                    "selector_width": record.counters().selector_width(),
                    "branch_width": record.counters().branch_width(),
                    "direct_snapshot_materialization_count": record.counters().direct_snapshot_materialization_count(),
                    "commit_envelope_materialization_count": record.counters().commit_envelope_materialization_count(),
                    "branch_head_materialization_count": record.counters().branch_head_materialization_count(),
                },
            }),
        )]),
        HarnessExecution::Stream(execution) => {
            stream_terminal_report_export::extensions_json(execution, runtime_bridge)
        }
        HarnessExecution::Source(execution) => {
            source_terminal_report_export::extensions_json(execution, runtime_bridge)
        }
        HarnessExecution::Policy(execution) => {
            policy_terminal_report_export::execution_extensions_json(execution, runtime_bridge)
        }
        HarnessExecution::Speculation(execution) => {
            speculation_terminal_report_export::extensions_json(execution, runtime_bridge)
        }
        HarnessExecution::Structural(execution) => {
            structural_terminal_report_export::extensions_json(execution, runtime_bridge)
        }
        HarnessExecution::Writeback(execution) => {
            writeback_terminal_report_export::extensions_json(execution)
        }
    }
}

fn route_execution_extensions_json(
    result: &crate::facade::BridgeRouteResult,
    continuity_summary: &Option<(
        crate::facade::BridgeContinuityArtifact,
        crate::facade::BridgeCanonicalContinuityRecord,
    )>,
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> BTreeMap<String, serde_json::Value> {
    BTreeMap::from([
        (
            "bridge_delivery_receipt".to_string(),
            json!({
                "delivered_target_count": result.receipt().delivered_target_count(),
                "snapshot_identity": result.receipt().snapshot_identity().as_str(),
            }),
        ),
        (
            "bridge_route_record".to_string(),
            runtime_bridge
                .diagnostics()
                .last_route_record()
                .map(route_record_json)
                .unwrap_or_else(|| json!(null)),
        ),
        (
            "bridge_continuity_record".to_string(),
            continuity_summary
                .as_ref()
                .map(|(artifact, canonical)| continuity_record_json(artifact, canonical))
                .unwrap_or_else(|| json!(null)),
        ),
    ])
}

fn continuity_record_json(
    artifact: &crate::facade::BridgeContinuityArtifact,
    canonical: &crate::facade::BridgeCanonicalContinuityRecord,
) -> serde_json::Value {
    json!({
        "route_identity": canonical.route_identity().as_str(),
        "continuity_request_digest": canonical.continuity_request_digest(),
        "continuity_resolution_digest": canonical.continuity_resolution_digest(),
        "continuity_artifact_identity": canonical.continuity_artifact_identity().as_str(),
        "source_snapshot": canonical.source_snapshot().as_str(),
        "source_branch": canonical.source_branch().as_str(),
        "outcome_classes": artifact
            .continuity_outcomes()
            .iter()
            .map(|entry| format!("{:?}", entry.outcome_class()))
            .collect::<Vec<_>>(),
    })
}
