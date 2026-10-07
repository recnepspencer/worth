use std::collections::BTreeMap;

use worth_harness::facade::{diagnostics_id, DiagnosticsRecord, ExecutionProfile, RunId};

use crate::facade::harness::RelationalHarnessError;
use crate::facade::runtime::RelationalRuntime;

use super::super::data::{RelationalFixture, RelationalHarnessAdapter};
use super::diagnostics_summary_fields::diagnostics_summary;

pub(super) fn capture_diagnostics(
    _adapter: &RelationalHarnessAdapter,
    runtime: &RelationalRuntime,
    _fixture: &worth_harness::facade::ScenarioFixture<RelationalFixture>,
    profile: &ExecutionProfile,
) -> Result<DiagnosticsRecord, RelationalHarnessError> {
    let run_id_value = RunId::new(format!("diagnostics:{}", profile.name));
    let publication_diagnostics = runtime.publication().diagnostic_access().snapshot();
    Ok(DiagnosticsRecord {
        schema_version: worth_harness::facade::RecordSchemaVersion::V1,
        diagnostics_id: diagnostics_id(&run_id_value),
        run_id: run_id_value,
        adapter_name: "worth-relational".to_string(),
        profile_name: profile.name.clone(),
        level: profile.diagnostics_level,
        time_marker: profile.time_marker.clone(),
        attachments: Vec::new(),
        summary: diagnostics_summary(
            profile.execution_mode,
            runtime.performance_access().counters(),
            publication_diagnostics,
        )
        .into_record_summary_value(),
        extensions: BTreeMap::new(),
    })
}
