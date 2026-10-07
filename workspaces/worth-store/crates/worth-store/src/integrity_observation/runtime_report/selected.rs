//! Version-two observations of Store-issued, protected selected C.5 records.
//! These have logical record identities, never raw artifact paths.
use super::{hex, outcome, vocabulary, PhysicalIntegrityRuntimeReportContext as Context};
use super::{PhysicalIntegrityRuntimeReportDenial as Denial, ReportWire};
use crate::physical_runtime::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubProgress as Progress,
    PhysicalIntegrityScrubSource,
};
use serde_json::json;
use std::io::Write;
use worth_store_physical_format::RootPublicationCell;

impl ManagedPhysicalIntegrityScrubHandle {
    /// Streams only actual selected-record windows under one protected C.5 root.
    /// Incomplete observations leave a partial sink document that must be discarded.
    /// The report grants no media, admission, or repair authority.
    pub fn write_selected_record_observation_report(
        &mut self,
        context: Context,
        maximum_report_bytes: u32,
        output: &mut impl Write,
    ) -> Result<u64, Denial> {
        if maximum_report_bytes == 0 || maximum_report_bytes > 16 * 1024 * 1024 {
            return Err(Denial::InvalidReportBound);
        }
        let executable = super::executable_identity()?;
        let initial = self.counters();
        let (store, deadline, remaining) = self.remaining_scope();
        let Some(first) = remaining.first() else {
            return Err(Denial::SelectedScopeRequired);
        };
        let Some(root) = first.issued_under().map(|observation| observation.root()) else {
            return Err(Denial::SelectedScopeRequired);
        };
        for target in remaining {
            if !matches!(
                target.source(),
                PhysicalIntegrityScrubSource::SelectedRecord(_)
            ) {
                return Err(Denial::SelectedScopeRequired);
            }
            if target.issued_under().map(|observation| observation.root()) != Some(root) {
                return Err(Denial::SelectedRootMismatch);
            }
        }
        let header = json!({
            "protocol":"store.physical.selected-integrity-observation", "version":2,
            "role":"runtime-selected-scrub", "executable":executable,
            "process":std::process::id().to_string(), "run":context.run,
            "scenario":context.scenario, "store":hex(&store.bytes()),
            "compatibility":{"earliest":2,"latest":2},
            "selected_root":root_identity(root),
            "declared_limits":{
                "entries":remaining.len(),
                "bytes":remaining.iter().map(|target| u64::from(target.declared_bytes())).sum::<u64>(),
                "window_bytes":remaining.iter().map(|target| target.declared_bytes()).max().unwrap_or(0),
                "elapsed_ms":deadline.as_millis(), "report_bytes":maximum_report_bytes
            }
        });
        let mut wire = ReportWire {
            output,
            written: 0,
            maximum: u64::from(maximum_report_bytes),
            bound_exhausted: false,
        };
        wire.append(b"{")?;
        wire.fields(&header)?;
        wire.append(b",\"artifacts\":[")?;
        let mut emitted = 0_u64;
        let mut validator_entries = 0_u64;
        loop {
            let target = self.remaining_scope().2.first().copied();
            match self.next_window() {
                Progress::WindowInspected(observation) => {
                    let target = target.expect("actual window has an admitted target");
                    if target.issued_under().map(|issuer| issuer.root()) != Some(root) {
                        return Err(Denial::SelectedRootMismatch);
                    }
                    if emitted != 0 {
                        wire.append(b",")?;
                    }
                    let record = match target.source() {
                        PhysicalIntegrityScrubSource::SelectedRecord(record) => record,
                        PhysicalIntegrityScrubSource::Media(_) => {
                            return Err(Denial::SelectedScopeRequired);
                        }
                    };
                    wire.value(&json!({
                        "family":vocabulary::family(observation.scope.artifact_family()),
                        "record":record_identity(record),
                        "outcome":outcome::project(observation.outcome)
                    }))?;
                    validator_entries += observation.validation_counters.inspected_frames();
                    emitted += 1;
                }
                Progress::Completed(_) | Progress::Indeterminate(_)
                    if self.remaining_scope().2.is_empty() =>
                {
                    break;
                }
                _ => return Err(Denial::ObservationIncomplete),
            }
        }
        wire.append(b"],\"completeness\":\"complete\",\"consumed\":{")?;
        let counters = self.counters();
        wire.fields(&json!({
            "entries":emitted,
            "bytes":counters.acquired_bytes - initial.acquired_bytes,
            "completed_windows":counters.completed_windows - initial.completed_windows,
            "validator_entries":validator_entries,
            "peak_allocation_bytes":counters.peak_allocation_bytes,
            "deferred_windows":counters.deferred_windows - initial.deferred_windows
        }))?;
        wire.append(b",\"report_bytes\":")?;
        let mut length = wire.written + 3;
        loop {
            let next = wire.written + 2 + length.to_string().len() as u64;
            if next == length {
                break;
            }
            length = next;
        }
        wire.append(length.to_string().as_bytes())?;
        wire.append(b"}}")?;
        Ok(wire.written)
    }
}

fn root_identity(root: RootPublicationCell) -> serde_json::Value {
    json!({"generation":root.generation().get(), "reference":root.root_reference().get()})
}

fn record_identity(record: worth_store_physical_format::PersistedRecordIdentity) -> String {
    let mut bytes = [0_u8; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    hex(&bytes)
}
