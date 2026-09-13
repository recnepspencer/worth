use std::fmt::Write;

use worth_foundational::PhysicalArtifactGeneration;

use super::report_wire_vocabulary::{
    blast, completeness, damage_cause, family, format_field, indeterminate, unknown,
    unsupported_axis,
};
use super::{
    OfflineArtifactDuplicateEvidence, OfflineIntegrityOutcome, OfflineIntegrityReport,
    PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY,
    PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineIntegrityReportWireDenial {
    ReportSizeExceeded { required: u64, maximum: u64 },
    CounterDidNotStabilize,
}

struct BoundedWire {
    wire: String,
    maximum: usize,
    required: u64,
}

struct RenderedWire {
    wire: String,
    required: u64,
}

pub fn encode_offline_integrity_report(
    report: &OfflineIntegrityReport,
) -> Result<String, OfflineIntegrityReportWireDenial> {
    let rendered = render(report);
    let required = rendered.required;
    let maximum = report.declared_limits().maximum_report_bytes();
    if required > maximum {
        return Err(OfflineIntegrityReportWireDenial::ReportSizeExceeded { required, maximum });
    }
    Ok(rendered.wire)
}

pub(crate) fn stabilize_report_bytes(
    report: &mut OfflineIntegrityReport,
) -> Result<(), OfflineIntegrityReportWireDenial> {
    for _ in 0..4 {
        let observed = render(report).required;
        if observed == report.counters().report_bytes() {
            return encode_offline_integrity_report(report).map(|_| ());
        }
        report.counters_mut().set_report_bytes(observed);
    }
    Err(OfflineIntegrityReportWireDenial::CounterDidNotStabilize)
}

fn render(report: &OfflineIntegrityReport) -> RenderedWire {
    let context = report.protocol_context();
    let limits = report.declared_limits();
    let counters = report.counters();
    let mut wire = BoundedWire::new(limits.maximum_report_bytes());
    wire.push('{');
    string_field_without_separator(
        &mut wire,
        "protocol",
        PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY.as_str(),
    );
    number_field(
        &mut wire,
        "version",
        PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION.get(),
    );
    string_field(&mut wire, "role", report.role_identity());
    string_field(&mut wire, "executable", context.executable_identity());
    string_field(&mut wire, "process", context.process_identity());
    string_field(&mut wire, "run", context.run_identity());
    string_field(&mut wire, "scenario", context.scenario_identity());
    optional_string_field(&mut wire, "store", report.store_identity());
    wire.push_str(",\"compatibility\":{\"earliest\":1,\"latest\":1}");
    wire.push_str(",\"declared_limits\":{");
    let declared = [
        ("entries", limits.maximum_entries()),
        ("bytes", limits.maximum_bytes()),
        ("open_files", u64::from(limits.maximum_open_files())),
        ("depth", u64::from(limits.maximum_depth())),
        ("symlinks", limits.maximum_symlinks()),
        ("elapsed_ms", limits.maximum_elapsed_milliseconds()),
        ("report_bytes", limits.maximum_report_bytes()),
    ];
    write_number_object(&mut wire, &declared);
    wire.push_str("},\"consumed\":{");
    let consumed = [
        ("entries", counters.entries_visited()),
        ("bytes", counters.bytes_read()),
        ("files_opened", counters.files_opened()),
        ("open_file_high_water", counters.open_file_high_water()),
        ("maximum_depth", counters.maximum_depth_reached()),
        ("symlinks_refused", counters.symlinks_refused()),
        ("duplicate_identities", counters.duplicate_identities()),
        ("missing_artifacts", counters.missing_artifacts()),
        ("unsupported_versions", counters.unsupported_versions()),
        ("indeterminate_reads", counters.indeterminate_reads()),
        ("exhausted_bounds", counters.exhausted_bounds()),
        ("checksum_calculations", counters.checksum_calculations()),
        (
            "namespace_identity_decoders",
            counters.namespace_identity_payload_decoder_entries(),
        ),
        (
            "durable_frame_decoders",
            counters.checksum_validated_durable_frames(),
        ),
        (
            "selector_decoders",
            counters.selector_payload_decoder_entries(),
        ),
        (
            "root_manifest_decoders",
            counters.root_manifest_payload_decoder_entries(),
        ),
        ("report_bytes", counters.report_bytes()),
    ];
    write_number_object(&mut wire, &consumed);
    wire.push_str("},");
    string_field_without_separator(
        &mut wire,
        "completeness",
        completeness(report.completeness()),
    );
    wire.push_str(",\"artifacts\":[");
    for (index, artifact) in report.artifacts().iter().enumerate() {
        if index != 0 {
            wire.push(',');
        }
        wire.push('{');
        string_field_without_separator(&mut wire, "path", artifact.relative_path());
        string_field(&mut wire, "family", family(artifact.family()));
        string_field(&mut wire, "identity", artifact.identity().as_str());
        match artifact.generation() {
            PhysicalArtifactGeneration::NotEncoded => wire.push_str(",\"generation\":null"),
            PhysicalArtifactGeneration::Encoded(value) => {
                number_field(&mut wire, "generation", value.get())
            }
        }
        if let Some(range) = artifact.range() {
            let _ = write!(
                wire,
                ",\"range\":{{\"offset\":{},\"length\":{}}}",
                range.offset(),
                range.length()
            );
        } else {
            wire.push_str(",\"range\":null");
        }
        duplicate_evidence(&mut wire, artifact.duplicates());
        outcome(&mut wire, artifact.outcome());
        wire.push('}');
    }
    wire.push_str("]}");
    wire.finish()
}

fn duplicate_evidence(wire: &mut BoundedWire, values: &[OfflineArtifactDuplicateEvidence]) {
    wire.push_str(",\"duplicates\":[");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            wire.push(',');
        }
        match value {
            OfflineArtifactDuplicateEvidence::PhysicalAlias { first_path } => {
                wire.push_str("{\"kind\":\"physical_alias\"");
                string_field(wire, "first_path", first_path);
                wire.push('}');
            }
            OfflineArtifactDuplicateEvidence::SemanticIdentity => {
                wire.push_str("{\"kind\":\"semantic_identity\"}");
            }
        }
    }
    wire.push(']');
}

fn outcome(wire: &mut BoundedWire, outcome: &OfflineIntegrityOutcome) {
    match outcome {
        OfflineIntegrityOutcome::Intact => wire.push_str(",\"outcome\":{\"posture\":\"intact\"}"),
        OfflineIntegrityOutcome::Damaged(localization) => {
            wire.push_str(",\"outcome\":{\"posture\":\"damaged\"");
            string_field(wire, "cause", damage_cause(localization.cause()));
            if let Some(range) = localization.damaged_range() {
                let _ = write!(
                    wire,
                    ",\"damaged_range\":{{\"offset\":{},\"length\":{}}}",
                    range.offset(),
                    range.length()
                );
            } else {
                wire.push_str(",\"damaged_range\":null");
            }
            optional_string_field(wire, "field", localization.field().map(format_field));
            string_field(wire, "blast_radius", blast(localization.blast_radius()));
            wire.push('}');
        }
        OfflineIntegrityOutcome::Unsupported(value) => {
            wire.push_str(",\"outcome\":{\"posture\":\"unsupported\"");
            string_field(wire, "axis", unsupported_axis(value.axis()));
            number_field(wire, "observed", value.observed());
            string_field(wire, "supported", value.supported());
            let range = value.range();
            let _ = write!(
                wire,
                ",\"range\":{{\"offset\":{},\"length\":{}}}}}",
                range.offset(),
                range.length()
            );
        }
        OfflineIntegrityOutcome::Unknown(reason) => {
            wire.push_str(",\"outcome\":{\"posture\":\"unknown\"");
            string_field(wire, "reason", unknown(*reason));
            wire.push('}');
        }
        OfflineIntegrityOutcome::Indeterminate(reason) => {
            wire.push_str(",\"outcome\":{\"posture\":\"indeterminate\"");
            string_field(wire, "reason", indeterminate(*reason));
            wire.push('}');
        }
    }
}

fn string_field(wire: &mut BoundedWire, name: &str, value: &str) {
    wire.push(',');
    string_field_without_separator(wire, name, value);
}

fn string_field_without_separator(wire: &mut BoundedWire, name: &str, value: &str) {
    let _ = write!(wire, "\"{}\":\"", name);
    escape_json(wire, value);
    wire.push('"');
}

fn optional_string_field(wire: &mut BoundedWire, name: &str, value: Option<&str>) {
    match value {
        Some(value) => string_field(wire, name, value),
        None => {
            let _ = write!(wire, ",\"{}\":null", name);
        }
    }
}

fn number_field(wire: &mut BoundedWire, name: &str, value: impl std::fmt::Display) {
    let _ = write!(wire, ",\"{}\":{}", name, value);
}

fn write_number_object(wire: &mut BoundedWire, values: &[(&str, u64)]) {
    for (index, (name, value)) in values.iter().enumerate() {
        if index != 0 {
            wire.push(',');
        }
        let _ = write!(wire, "\"{}\":{}", name, value);
    }
}

fn escape_json(wire: &mut BoundedWire, value: &str) {
    for character in value.chars() {
        match character {
            '"' => wire.push_str("\\\""),
            '\\' => wire.push_str("\\\\"),
            '\n' => wire.push_str("\\n"),
            '\r' => wire.push_str("\\r"),
            '\t' => wire.push_str("\\t"),
            value if value.is_control() => {
                let _ = write!(wire, "\\u{:04x}", value as u32);
            }
            value => wire.push(value),
        }
    }
}

impl BoundedWire {
    fn new(maximum: u64) -> Self {
        Self {
            wire: String::new(),
            maximum: usize::try_from(maximum).unwrap_or(usize::MAX),
            required: 0,
        }
    }

    fn push_str(&mut self, value: &str) {
        self.required = self.required.saturating_add(value.len() as u64);
        let remaining = self.maximum.saturating_sub(self.wire.len());
        let mut accepted = remaining.min(value.len());
        while accepted > 0 && !value.is_char_boundary(accepted) {
            accepted -= 1;
        }
        self.wire.push_str(&value[..accepted]);
    }

    fn push(&mut self, value: char) {
        self.required = self.required.saturating_add(value.len_utf8() as u64);
        if self.wire.len().saturating_add(value.len_utf8()) <= self.maximum {
            self.wire.push(value);
        }
    }

    fn finish(self) -> RenderedWire {
        RenderedWire {
            wire: self.wire,
            required: self.required,
        }
    }
}

impl std::fmt::Write for BoundedWire {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.push_str(value);
        Ok(())
    }
}
