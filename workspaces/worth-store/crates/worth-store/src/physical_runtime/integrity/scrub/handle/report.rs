use super::{
    ManagedPhysicalIntegrityScrubHandle, ManagedPhysicalIntegrityScrubProgress as Progress,
};
use serde_json::{json, Value};
use std::io::Write;

mod artifact;
mod outcome;
mod vocabulary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalIntegrityRuntimeReportDenial {
    InvalidIdentity,
    ExecutableIdentityUnavailable,
    InvalidReportBound,
    ReportBoundExceeded,
    SinkWriteFailure,
}

/// Descriptive correlation only. Process/executable identity is sampled by the
/// runtime; neither these labels nor emitted JSON grants admission authority.
pub struct PhysicalIntegrityRuntimeReportContext {
    run: Box<str>,
    scenario: Box<str>,
}
impl PhysicalIntegrityRuntimeReportContext {
    pub fn new(run: &str, scenario: &str) -> Result<Self, PhysicalIntegrityRuntimeReportDenial> {
        for value in [run, scenario] {
            if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
                return Err(PhysicalIntegrityRuntimeReportDenial::InvalidIdentity);
            }
        }
        Ok(Self {
            run: run.into(),
            scenario: scenario.into(),
        })
    }
}

impl ManagedPhysicalIntegrityScrubHandle {
    /// Streams actual remaining windows as bounded version-1 JSON to a caller
    /// sink. No report buffer is retained or allocated by Store. Source-window
    /// resources are released before each sink write. Stops at the first
    /// deferral/pause/terminal, without retry. A failed sink/bound may contain an
    /// incomplete document: discard it; completed cursor/counters stay truthful.
    pub fn write_observation_report(
        &mut self,
        context: PhysicalIntegrityRuntimeReportContext,
        maximum_report_bytes: u32,
        output: &mut impl Write,
    ) -> Result<u64, PhysicalIntegrityRuntimeReportDenial> {
        use PhysicalIntegrityRuntimeReportDenial as Denial;
        if maximum_report_bytes == 0 || maximum_report_bytes > 16 * 1024 * 1024 {
            return Err(Denial::InvalidReportBound);
        }
        let executable = std::env::current_exe()
            .ok()
            .and_then(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .filter(|name| {
                !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
            })
            .ok_or(Denial::ExecutableIdentityUnavailable)?;
        let initial = self.counters;
        let remaining = &self.request.targets[self.next_target.min(self.request.targets.len())..];
        let header = json!({"protocol":"store.physical.integrity-observation", "version":1,
            "role":"runtime-integrity-observer", "executable":executable, "process":std::process::id().to_string(),
            "run":context.run, "scenario":context.scenario, "store":hex(&self.request.store.bytes()),
            "compatibility":{"earliest":1,"latest":1},
            "declared_limits":{"entries":remaining.len(), "bytes":remaining.iter().map(|target| u64::from(target.range().length())).sum::<u64>(),
                "window_bytes":remaining.iter().map(|target| target.range().length()).max().unwrap_or(0),
                "elapsed_ms":self.request.deadline.as_millis(), "report_bytes":maximum_report_bytes}});
        let mut wire = ReportWire {
            output,
            written: 0,
            maximum: u64::from(maximum_report_bytes),
            bound_exhausted: false,
        };
        wire.append(b"{")?;
        wire.fields(&header)?;
        wire.append(b",\"artifacts\":[")?;
        let (emitted, validator_entries, completeness) = self.write_windows(&mut wire)?;
        wire.append(b"],\"completeness\":")?;
        wire.value(&json!(completeness))?;
        wire.append(b",\"consumed\":{")?;
        let counters = self.counters;
        wire.fields(&json!({"entries":emitted, "bytes":counters.acquired_bytes - initial.acquired_bytes,
            "completed_windows":counters.completed_windows - initial.completed_windows,
            "validator_entries":validator_entries, "checksum_calculations":null, "owner_decoder_entries":0,
            "peak_allocation_bytes":counters.peak_allocation_bytes,
            "deferred_windows":counters.deferred_windows - initial.deferred_windows}))?;
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

    fn write_windows<W: Write>(
        &mut self,
        wire: &mut ReportWire<'_, W>,
    ) -> Result<(u64, u64, &'static str), PhysicalIntegrityRuntimeReportDenial> {
        let mut emitted = 0;
        let mut validator_entries = 0;
        let completeness = loop {
            let target = self.request.targets.get(self.next_target).copied();
            match self.next_window() {
                Progress::WindowInspected(observation) => {
                    if emitted != 0 {
                        wire.append(b",")?;
                    }
                    wire.value(&artifact::project(
                        target.expect("actual window has an admitted target"),
                        &observation,
                    ))?;
                    validator_entries += observation.validation_counters.inspected_frames();
                    emitted += 1;
                }
                Progress::Completed(_) => break "complete",
                Progress::DeadlineExceeded(_) => break "bound_exhausted",
                _ => break "indeterminate",
            }
        };
        Ok((emitted, validator_entries, completeness))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

struct ReportWire<'sink, W> {
    output: &'sink mut W,
    written: u64,
    maximum: u64,
    bound_exhausted: bool,
}
impl<W: Write> ReportWire<'_, W> {
    fn denial(&self) -> PhysicalIntegrityRuntimeReportDenial {
        if self.bound_exhausted {
            PhysicalIntegrityRuntimeReportDenial::ReportBoundExceeded
        } else {
            PhysicalIntegrityRuntimeReportDenial::SinkWriteFailure
        }
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), PhysicalIntegrityRuntimeReportDenial> {
        self.write_all(bytes).map_err(|_| self.denial())
    }
    fn value(&mut self, value: &Value) -> Result<(), PhysicalIntegrityRuntimeReportDenial> {
        serde_json::to_writer(&mut *self, value).map_err(|_| self.denial())
    }
    fn fields(&mut self, value: &Value) -> Result<(), PhysicalIntegrityRuntimeReportDenial> {
        for (index, (key, value)) in value
            .as_object()
            .expect("fixed report object")
            .iter()
            .enumerate()
        {
            if index != 0 {
                self.append(b",")?;
            }
            self.value(&json!(key))?;
            self.append(b":")?;
            self.value(value)?;
        }
        Ok(())
    }
}
impl<W: Write> Write for ReportWire<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() as u64 > self.maximum.saturating_sub(self.written) {
            self.bound_exhausted = true;
            return Err(std::io::ErrorKind::OutOfMemory.into());
        }
        let written = self.output.write(bytes)?;
        self.written += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}
