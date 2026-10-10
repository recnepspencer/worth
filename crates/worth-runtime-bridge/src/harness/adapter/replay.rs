use worth_harness::facade::{
    replay_id, HarnessAdapter, RecordSchemaVersion, ReplayHarnessAdapter, ReplayRecord,
    ReplayRequest, ScenarioFixture,
};

use super::target_id::BridgeHarnessTargetId;
use super::terminal_report_export::{historical_replay_summary_json, route_replay_summary_json};
use super::types::{BridgeHarnessAdapter, BridgeHarnessError, BridgeHarnessSession};

impl ReplayHarnessAdapter for BridgeHarnessAdapter {
    fn capture_replay(
        &self,
        runtime: &BridgeHarnessSession,
        fixture: &ScenarioFixture<crate::harness::fixtures::BridgeHarnessFixture>,
        replay: &ReplayRequest<BridgeHarnessTargetId>,
    ) -> Result<ReplayRecord<BridgeHarnessTargetId>, BridgeHarnessError> {
        crate::host_execution::with_declared_request(
            fixture.fixture.policy().execution(),
            |resource_request| {
                let runtime_bridge = runtime
                    .runtime
                    .as_ref()
                    .ok_or_else(|| BridgeHarnessError::new("bridge runtime not loaded"))?;
                if let Some(record) = runtime_bridge
                    .diagnostics()
                    .last_historical_evaluation_record()
                {
                    let replay_record = runtime_bridge
                        .replay_canonical_historical_evaluation_record(&record, resource_request)
                        .map_err(|error| {
                            BridgeHarnessError::new(format!(
                                "bridge historical replay failed: {error}"
                            ))
                        })?;
                    return Ok(ReplayRecord {
                        schema_version: RecordSchemaVersion::V1,
                        replay_id: replay_id(&replay.source_run.run_id, &replay.name),
                        source_run_id: replay.source_run.run_id.clone(),
                        scenario_id: worth_harness::facade::scenario_id(&fixture.name),
                        adapter_name: self.adapter_name().to_string(),
                        scenario_name: fixture.name.clone(),
                        replay_name: replay.name.clone(),
                        requested_targets: replay.request.targets.clone(),
                        summary: historical_replay_summary_json(&replay_record),
                        attachments: Vec::new(),
                    });
                }

                let route_record = runtime_bridge
                    .diagnostics()
                    .last_canonical_route_record()
                    .ok_or_else(|| {
                        BridgeHarnessError::new("no canonical bridge route record available")
                    })?;
                let replay_record = runtime_bridge
                    .replay_canonical_record(&route_record, resource_request)
                    .map_err(|error| {
                        BridgeHarnessError::new(format!("bridge replay failed: {error}"))
                    })?;
                Ok(ReplayRecord {
                    schema_version: RecordSchemaVersion::V1,
                    replay_id: replay_id(&replay.source_run.run_id, &replay.name),
                    source_run_id: replay.source_run.run_id.clone(),
                    scenario_id: worth_harness::facade::scenario_id(&fixture.name),
                    adapter_name: self.adapter_name().to_string(),
                    scenario_name: fixture.name.clone(),
                    replay_name: replay.name.clone(),
                    requested_targets: replay.request.targets.clone(),
                    summary: route_replay_summary_json(&replay_record),
                    attachments: Vec::new(),
                })
            },
        )
    }
}
