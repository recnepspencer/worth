//! Project domain execution observations into harness records.

use std::collections::BTreeMap;

use serde_json::{json, Value};
use worth_harness::facade::{
    AttachmentRecord, ExecutionProfile, ExecutionRequest, RecordSchemaVersion, RunOutcome,
    RunRecord, RunStatus, ScenarioFixture, SnapshotObservation, SnapshotPayload, SnapshotRecord,
    StructuredValue, TargetStatusRecord,
};

use crate::data::core_profile::CORE_STORAGE_PROFILE_ID;
use crate::data::error::SignalError;
use crate::diagnostics::policy::DiagnosticsAvailability;
use crate::logic::explain::NodeExplanation;
use crate::logic::planner::{ExecutionReport, PlanSummary};
use crate::runtime_policy::SignalRuntimePolicy;

use super::super::runtime::{SignalFixtureFactory, SignalHarnessRuntime};
use super::SignalHarnessBridge;

pub(super) struct RunProjection<'a> {
    pub adapter_name: &'a str,
    pub fixture: &'a ScenarioFixture<SignalFixtureFactory>,
    pub request: &'a ExecutionRequest<String>,
    pub profile: &'a ExecutionProfile,
    pub plan_summary: PlanSummary,
    pub report: &'a ExecutionReport,
    pub runtime_policy: SignalRuntimePolicy,
    pub target_statuses: Vec<TargetStatusRecord>,
}

impl SignalHarnessBridge {
    pub(in crate::presentation::harness) fn runtime_policy_summary(
        policy: SignalRuntimePolicy,
    ) -> Value {
        json!({
            "profile": format!("{:?}", policy.tier),
            "history_limit": policy.retention_budget.history_limit,
            "detail_limit": policy.retention_budget.detail_limit,
            "retain_history_details": policy.retention_budget.retain_history_details,
            "retain_flow_explanation": policy.retention_budget.retain_flow_explanation,
            "retain_latest_failure_context": policy.retention_budget.retain_latest_failure_context,
            "retain_stage_details": policy.retention_budget.retain_stage_details,
            "capture_forensic_failure_context": policy.retention_budget.capture_forensic_failure_context,
            "explanation_retention": format!("{:?}", policy.retention_budget.explanation_retention),
            "provenance_retention": format!("{:?}", policy.retention_budget.provenance_retention),
            "replay_detail": format!("{:?}", policy.retention_budget.replay_detail),
            "semantic_retention": format!("{:?}", policy.retention_budget.semantic_detail),
            "parallel_admission": {
                "throughput_min_parallel_tasks": policy.parallel_admission.throughput_min_parallel_tasks,
                "balanced_min_parallel_tasks": policy.parallel_admission.balanced_min_parallel_tasks,
                "latency_bounded_min_parallel_tasks": policy.parallel_admission.latency_bounded_min_parallel_tasks,
                "full_parallel_min_tasks": policy.parallel_admission.full_parallel_min_tasks,
            },
        })
    }

    pub(in crate::presentation::harness) fn artifact_materialization_label(
        mode: DiagnosticsAvailability,
    ) -> &'static str {
        match mode {
            DiagnosticsAvailability::RetainedAvailable => "retained",
            DiagnosticsAvailability::ReconstructedAvailable => "reconstructed",
            DiagnosticsAvailability::OmittedByTier => "omitted_by_tier",
            DiagnosticsAvailability::DeniedByBudget => "denied_by_budget",
            DiagnosticsAvailability::ObservationNotActivated => "observation_not_activated",
            DiagnosticsAvailability::UnavailableNotRetained => "unavailable_not_retained",
            DiagnosticsAvailability::UnavailableNotReconstructable => {
                "unavailable_not_reconstructable"
            }
        }
    }

    pub(in crate::presentation::harness) fn explanation_summary(
        explanation: &NodeExplanation,
    ) -> Value {
        json!({
            "node": explanation.node.to_string(),
            "state": format!("{:?}", explanation.state),
            "contract_reads_mask": explanation.contract_reads.bits(),
            "contract_produces_mask": explanation.contract_produces.bits(),
            "contract_partition_scope_count": explanation.contract_partition_scope.as_ref().map(|scopes| scopes.len()).unwrap_or(0),
            "required_context": format!("{:?}", explanation.required_context),
            "execution_record_id": explanation.execution_record_id,
            "semantic_segment_id": explanation.semantic_segment_id,
            "upstream_count": explanation.upstream.len(),
            "propagation_suppressed": explanation.propagation_suppressed,
            "output_change": explanation.output_change.map(|change| format!("{change:?}")),
        })
    }

    pub(super) fn report_summary(report: &ExecutionReport) -> Value {
        json!({
            "stage_count": report.stage_count,
            "task_count": report.task_count,
            "tasks_executed": report.tasks_executed,
            "tasks_pruned": report.tasks_pruned,
            "tasks_validated_clean": report.tasks_validated_clean,
            "tasks_deferred_by_condition": report.tasks_deferred_by_condition,
            "tasks_reverted_clean_by_condition": report.tasks_reverted_clean_by_condition,
            "tasks_satisfied_by_memoization": report.tasks_satisfied_by_memoization,
            "tasks_with_suppressed_propagation": report.tasks_with_suppressed_propagation,
            "core_storage_profile": CORE_STORAGE_PROFILE_ID,
        })
    }

    pub(super) fn target_statuses(
        runtime: &SignalHarnessRuntime,
        targets: &[String],
    ) -> Result<Vec<TargetStatusRecord>, SignalError> {
        targets
            .iter()
            .map(|label| {
                let node = runtime.resolve(label)?;
                let state = runtime.graph.get_state(node)?;
                Ok(TargetStatusRecord {
                    target: label.clone(),
                    status: Self::observation_status(state),
                    detail: Some(format!("{state:?}")),
                })
            })
            .collect()
    }

    pub(super) fn run_record(input: RunProjection<'_>) -> RunRecord<String> {
        let scenario_id = worth_harness::facade::scenario_id(&input.fixture.name);
        let run_id =
            worth_harness::facade::run_id(&scenario_id, &input.profile.name, &input.request.name);
        let plan_summary = serde_json::to_value(input.plan_summary).unwrap_or_else(|_| json!({}));

        RunRecord {
            schema_version: RecordSchemaVersion::V2,
            run_id: run_id.clone(),
            scenario_id,
            adapter_name: input.adapter_name.to_string(),
            scenario_name: input.fixture.name.clone(),
            profile_name: input.profile.name.clone(),
            time_marker: input.profile.time_marker.clone(),
            feed_batch: input.request.feed_batch.clone(),
            execution_mode: input.profile.execution_mode,
            diagnostics_level: input.profile.diagnostics_level,
            status: RunStatus::Succeeded,
            outcome: RunOutcome::Completed,
            budget_usage: None,
            requested_targets: input.request.targets.clone(),
            target_statuses: input.target_statuses,
            changed_targets: input.request.targets.clone(),
            attachments: vec![AttachmentRecord::json(
                "evaluation-plan-summary",
                plan_summary.clone(),
            )],
            summary: Self::report_summary(input.report),
            extensions: BTreeMap::from([
                ("evaluation_plan_summary".to_string(), plan_summary),
                (
                    "execution_report".to_string(),
                    serde_json::to_value(input.report).unwrap_or_else(|_| json!({})),
                ),
                (
                    "stage_parallel_admission".to_string(),
                    json!(input
                        .report
                        .stages
                        .iter()
                        .map(|stage| {
                            let serial_apply_rejection_reason = stage
                                .serial_apply_rejection_reason
                                .map(|reason| reason.code());
                            json!({
                                "stage_index": stage.stage_index,
                                "reason": stage.parallel_admission_reason.map(|reason| reason.code()),
                                "message": stage.parallel_admission_message(),
                                "serial_apply_rejection_reason": serial_apply_rejection_reason,
                            })
                        })
                        .collect::<Vec<_>>()),
                ),
                (
                    "runtime_policy".to_string(),
                    Self::runtime_policy_summary(input.runtime_policy),
                ),
                (
                    "core_storage_profile".to_string(),
                    json!(CORE_STORAGE_PROFILE_ID),
                ),
            ]),
        }
    }

    pub(super) fn snapshot_record(
        adapter_name: &str,
        runtime: &SignalHarnessRuntime,
        fixture: &ScenarioFixture<SignalFixtureFactory>,
        request: &ExecutionRequest<String>,
        profile: &ExecutionProfile,
    ) -> Result<SnapshotRecord<String>, SignalError> {
        let scenario_id = worth_harness::facade::scenario_id(&fixture.name);
        let run_id = worth_harness::facade::run_id(&scenario_id, &profile.name, &request.name);
        let observations = request
            .targets
            .iter()
            .map(|label| {
                let node = runtime.resolve(label)?;
                let state = runtime.graph.get_state(node)?;
                Ok(SnapshotObservation {
                    target: label.clone(),
                    status: Self::observation_status(state),
                    detail: Some(format!("{state:?}")),
                    value: Some(SnapshotPayload::Structured(StructuredValue::Json(json!({
                        "node": node.to_string(),
                        "state": format!("{state:?}"),
                    })))),
                })
            })
            .collect::<Result<Vec<_>, SignalError>>()?;

        Ok(SnapshotRecord {
            schema_version: RecordSchemaVersion::V2,
            snapshot_id: worth_harness::facade::snapshot_id(&run_id, "capture"),
            run_id,
            adapter_name: adapter_name.to_string(),
            scenario_name: fixture.name.clone(),
            profile_name: profile.name.clone(),
            time_marker: profile.time_marker.clone(),
            observations,
            attachments: Vec::new(),
            extensions: BTreeMap::new(),
        })
    }
}
