use std::collections::BTreeMap;

use worth_harness::facade::{
    run_id, ExecutionProfile, ExecutionRequest, RunOutcome, RunRecord, RunStatus,
    TargetStatusRecord,
};

use crate::facade::harness::RelationalHarnessError;
use crate::facade::runtime::RelationalRuntime;
use crate::query::data::PlannedQueryPacket;

use super::super::data::{RelationalFixture, RelationalHarnessAdapter};
use super::super::targets::{parse_target, resolve_targets};
use super::run_summary_fields::{publication_artifacts_extension, run_summary};

pub(super) fn prepare_runtime(
    _runtime: &mut RelationalRuntime,
    _profile: &ExecutionProfile,
) -> Result<(), RelationalHarnessError> {
    Ok(())
}

pub(super) fn execute_request(
    _adapter: &RelationalHarnessAdapter,
    runtime: &RelationalRuntime,
    fixture: &worth_harness::facade::ScenarioFixture<RelationalFixture>,
    request: &ExecutionRequest<String>,
    profile: &ExecutionProfile,
) -> Result<RunRecord<String>, RelationalHarnessError> {
    let scenario_id_value = worth_harness::facade::scenario_id(&fixture.name);
    let run_id_value = run_id(&scenario_id_value, &profile.name, &request.name);
    let snapshot = runtime.visibility_authority().snapshot();
    let targets = resolve_targets(request);
    let parsed_targets = targets
        .iter()
        .map(|target| parse_target(target))
        .collect::<Result<Vec<_>, _>>()?;
    let context = runtime
        .read_truth()
        .query_plan_context(&snapshot)
        .ok_or_else(|| RelationalHarnessError("query plan context unavailable".to_string()))?;
    let result = runtime
        .read_truth()
        .execute_query_plan(
            runtime
                .read_truth()
                .plan_query_packet(
                    &snapshot,
                    PlannedQueryPacket::explicit_targets("execute", context, parsed_targets),
                )
                .ok_or_else(|| RelationalHarnessError("query plan unavailable".to_string()))?,
        )
        .ok_or_else(|| RelationalHarnessError("query execution unavailable".to_string()))?
        .result;
    let publication_artifacts = runtime.publication().artifacts().snapshot();
    Ok(RunRecord {
        schema_version: worth_harness::facade::RecordSchemaVersion::V1,
        run_id: run_id_value,
        scenario_id: scenario_id_value,
        adapter_name: "worth-relational".to_string(),
        scenario_name: fixture.name.clone(),
        profile_name: profile.name.clone(),
        time_marker: profile.time_marker.clone(),
        feed_batch: request.feed_batch.clone(),
        execution_mode: profile.execution_mode,
        diagnostics_level: profile.diagnostics_level,
        status: RunStatus::Succeeded,
        outcome: RunOutcome::Completed,
        budget_usage: None,
        requested_targets: targets.clone(),
        target_statuses: targets
            .iter()
            .map(|target: &String| TargetStatusRecord {
                target: target.clone(),
                status: worth_harness::facade::ObservationStatus::Validated,
                detail: None,
            })
            .collect(),
        changed_targets: targets,
        attachments: Vec::new(),
        summary: run_summary(&snapshot, result.entities.len(), result.relations.len())
            .into_record_summary_value(),
        extensions: BTreeMap::from([(
            "publication_artifacts".to_string(),
            publication_artifacts_extension(publication_artifacts).into_record_summary_value(),
        )]),
    })
}
