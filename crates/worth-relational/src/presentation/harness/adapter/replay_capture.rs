use worth_harness::facade::{replay_id, ReplayRecord, ReplayRequest};

use crate::facade::harness::RelationalHarnessError;
use crate::facade::runtime::RelationalRuntime;

use super::super::data::{RelationalFixture, RelationalHarnessAdapter};
use super::replay_summary_fields::replay_summary;

pub(super) fn capture_replay(
    _adapter: &RelationalHarnessAdapter,
    runtime: &RelationalRuntime,
    fixture: &worth_harness::facade::ScenarioFixture<RelationalFixture>,
    replay: &ReplayRequest<String>,
) -> Result<ReplayRecord<String>, RelationalHarnessError> {
    let (position, _) = runtime.history.latest_canonical_patch_route()
        .ok_or_else(|| RelationalHarnessError("no replay artifact available".to_string()))?;
    let envelope = runtime.history.canonical_envelope_at_patch(position)
        .ok_or_else(|| RelationalHarnessError("canonical replay envelope unavailable".to_string()))?;
    let snapshot_id = runtime.visibility.published_snapshot_binding_for_version(envelope.commit.version_id)
        .map(|(id, _)| id);
    Ok(ReplayRecord {
        schema_version: worth_harness::facade::RecordSchemaVersion::V1,
        replay_id: replay_id(&replay.source_run.run_id, &replay.name),
        source_run_id: replay.source_run.run_id.clone(),
        scenario_id: worth_harness::facade::scenario_id(&fixture.name),
        adapter_name: "worth-relational".to_string(),
        scenario_name: fixture.name.clone(),
        replay_name: replay.name.clone(),
        requested_targets: replay.request.targets.clone(),
        summary: replay_summary(&envelope, position, snapshot_id).into_record_summary_value(),
        attachments: Vec::new(),
    })
}
