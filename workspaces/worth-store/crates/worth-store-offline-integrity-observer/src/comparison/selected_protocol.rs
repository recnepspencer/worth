//! Strict, path-independent selected-record comparison input. Version one is
//! deliberately parsed by its existing path-keyed module, not this protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use worth_store_physical_format::PersistedRecordIdentity;

use super::{
    protocol::{Compatibility, Outcome},
    PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits,
};

pub(super) const PROTOCOL: &str = "store.physical.selected-integrity-observation";
pub(super) const VERSION: u32 = 2;
pub(super) const RUNTIME_ROLE: &str = "runtime-selected-scrub";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SelectedReport {
    pub protocol: String,
    pub version: u32,
    pub role: String,
    pub executable: String,
    pub process: String,
    pub run: String,
    pub scenario: String,
    pub store: String,
    pub compatibility: Compatibility,
    pub declared_limits: Value,
    pub consumed: Value,
    pub completeness: String,
    pub selected_root: SelectedRoot,
    pub artifacts: Vec<SelectedArtifact>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct SelectedRoot {
    pub generation: u64,
    pub reference: u64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SelectedArtifact {
    pub family: String,
    pub record: String,
    pub outcome: Outcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_path: Option<String>,
}

pub(super) fn parse(
    wire: &str,
    role: &str,
    limits: PhysicalIntegrityComparisonLimits,
) -> Result<SelectedReport, Denial> {
    if wire.len() as u64 > limits.maximum_input_bytes() {
        return Err(Denial::InputBoundExceeded);
    }
    let report: SelectedReport =
        serde_json::from_str(wire).map_err(|_| Denial::MalformedProtocol)?;
    if report.protocol != PROTOCOL || report.version != VERSION {
        return Err(Denial::UnsupportedProtocol);
    }
    if report.role != role {
        return Err(Denial::InvalidRole);
    }
    if report.compatibility.earliest != VERSION || report.compatibility.latest != VERSION {
        return Err(Denial::InvalidCompatibility);
    }
    for value in [
        &report.executable,
        &report.process,
        &report.run,
        &report.scenario,
    ] {
        short_identity(value)?;
    }
    if !lower_hex(&report.store, 32) {
        return Err(Denial::InvalidIdentity);
    }
    if report.selected_root.generation == 0
        || report.selected_root.reference != report.selected_root.generation
    {
        return Err(Denial::SelectedRootUnavailable);
    }
    if report.completeness != "complete" {
        return Err(Denial::IncompleteObservation);
    }
    if !report.declared_limits.is_object() || !report.consumed.is_object() {
        return Err(Denial::InvalidObservation);
    }
    if report.artifacts.len() as u64 > limits.maximum_artifacts() {
        return Err(Denial::ArtifactBoundExceeded);
    }
    let mut seen = BTreeSet::new();
    for artifact in &report.artifacts {
        if family(&artifact.family).is_none() {
            return Err(Denial::InvalidObservation);
        }
        if !seen.insert(record(&artifact.record)?) {
            return Err(Denial::DuplicateScope);
        }
        if report.role == RUNTIME_ROLE && artifact.physical_path.is_some() {
            return Err(Denial::InvalidObservation);
        }
        if report.role != RUNTIME_ROLE && artifact.physical_path.is_none() {
            return Err(Denial::InvalidObservation);
        }
        if let Some(path) = &artifact.physical_path {
            if path.is_empty() || path.len() > 32768 || path.chars().any(char::is_control) {
                return Err(Denial::InvalidObservation);
            }
        }
        validate_outcome(&artifact.outcome)?;
    }
    Ok(report)
}

pub(super) fn family(value: &str) -> Option<worth_foundational::PhysicalArtifactFamily> {
    use worth_foundational::PhysicalArtifactFamily;
    Some(match value {
        "blob_chunk_frame" => PhysicalArtifactFamily::BlobChunkFrame,
        "blob_tree_node" => PhysicalArtifactFamily::BlobTreeNode,
        "blob_generation_publication" => PhysicalArtifactFamily::BlobGenerationPublication,
        "btree_node" => PhysicalArtifactFamily::BTreeNode,
        _ => return None,
    })
}

pub(super) fn record(value: &str) -> Result<PersistedRecordIdentity, Denial> {
    if !lower_hex(value, 48) {
        return Err(Denial::InvalidRecordIdentity);
    }
    let mut raw = [0_u8; 24];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        raw[index] = (hex_digit(pair[0]) << 4) | hex_digit(pair[1]);
    }
    PersistedRecordIdentity::new(
        raw[..16].try_into().expect("fixed record epoch"),
        u64::from_le_bytes(raw[16..].try_into().expect("fixed record ordinal")),
    )
    .ok_or(Denial::InvalidRecordIdentity)
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_digit(byte: u8) -> u8 {
    if byte.is_ascii_digit() {
        byte - b'0'
    } else {
        byte - b'a' + 10
    }
}

fn short_identity(value: &str) -> Result<(), Denial> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        Err(Denial::InvalidIdentity)
    } else {
        Ok(())
    }
}

fn valid_range(range: &super::protocol::Range) -> bool {
    worth_foundational::PhysicalByteRange::new(range.offset, range.length).is_ok()
}

fn validate_outcome(outcome: &Outcome) -> Result<(), Denial> {
    let valid = match outcome {
        Outcome::Intact => true,
        Outcome::Damaged {
            cause,
            damaged_range,
            field,
            blast_radius,
        } => {
            short_identity(cause)?;
            short_identity(blast_radius)?;
            if let Some(field) = field {
                short_identity(field)?;
            }
            damaged_range.as_ref().is_none_or(valid_range)
        }
        Outcome::Unsupported {
            axis,
            supported,
            range,
            ..
        } => {
            short_identity(axis)?;
            short_identity(supported)?;
            valid_range(range)
        }
        Outcome::Unknown { reason } => {
            short_identity(reason)?;
            true
        }
        Outcome::Indeterminate {
            reason,
            observed_range,
        } => {
            short_identity(reason)?;
            observed_range.as_ref().is_none_or(valid_range)
        }
    };
    valid.then_some(()).ok_or(Denial::InvalidObservation)
}
