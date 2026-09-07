use super::{PhysicalIntegrityComparisonDenial as Denial, PhysicalIntegrityComparisonLimits};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use worth_foundational::{PhysicalArtifactIdentity, PhysicalByteRange, PhysicalIntegrityPosture};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObservationReport {
    pub protocol: String,
    pub version: u32,
    pub role: String,
    pub executable: String,
    pub process: String,
    pub run: String,
    pub scenario: String,
    pub store: Option<String>,
    pub compatibility: Compatibility,
    pub declared_limits: Value,
    pub consumed: Value,
    pub completeness: String,
    pub artifacts: Vec<Artifact>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Compatibility {
    pub earliest: u32,
    pub latest: u32,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Artifact {
    pub path: String,
    pub family: String,
    pub identity: String,
    pub generation: Option<u64>,
    pub range: Option<Range>,
    pub duplicates: Vec<Value>,
    pub outcome: Outcome,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Range {
    pub offset: u64,
    pub length: u64,
}
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "posture", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum Outcome {
    Intact,
    Damaged {
        cause: String,
        damaged_range: Option<Range>,
        field: Option<String>,
        blast_radius: String,
    },
    Unsupported {
        axis: String,
        observed: u64,
        supported: String,
        range: Range,
    },
    Unknown {
        reason: String,
    },
    Indeterminate {
        reason: String,
    },
}

pub(super) fn parse(
    wire: &str,
    role: &str,
    limits: PhysicalIntegrityComparisonLimits,
) -> Result<ObservationReport, Denial> {
    if wire.len() as u64 > limits.input_bytes {
        return Err(Denial::InputBoundExceeded);
    }
    let report: ObservationReport =
        serde_json::from_str(wire).map_err(|_| Denial::MalformedProtocol)?;
    if report.protocol != crate::PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_IDENTITY.as_str()
        || report.version != crate::PHYSICAL_INTEGRITY_OBSERVATION_PROTOCOL_VERSION.get()
    {
        return Err(Denial::UnsupportedProtocol);
    }
    if report.role != role {
        return Err(Denial::InvalidRole);
    }
    if report.compatibility.earliest == 0
        || report.compatibility.earliest > report.version
        || report.compatibility.latest < report.version
    {
        return Err(Denial::InvalidCompatibility);
    }
    for value in [
        &report.executable,
        &report.process,
        &report.run,
        &report.scenario,
    ] {
        identity(value)?;
    }
    if let Some(store) = &report.store {
        identity(store)?;
    }
    if !matches!(
        report.completeness.as_str(),
        "complete" | "bound_exhausted" | "indeterminate"
    ) {
        return Err(Denial::InvalidObservation);
    }
    if report.artifacts.len() as u64 > limits.artifacts {
        return Err(Denial::ArtifactBoundExceeded);
    }
    if !report.declared_limits.is_object() || !report.consumed.is_object() {
        return Err(Denial::InvalidObservation);
    }
    for artifact in &report.artifacts {
        validate_artifact(artifact)?;
    }
    Ok(report)
}

fn identity(value: &str) -> Result<(), Denial> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        Err(Denial::InvalidIdentity)
    } else {
        Ok(())
    }
}
fn range(value: &Range) -> Result<(), Denial> {
    PhysicalByteRange::new(value.offset, value.length)
        .map(|_| ())
        .map_err(|_| Denial::InvalidObservation)
}
fn validate_artifact(artifact: &Artifact) -> Result<(), Denial> {
    PhysicalArtifactIdentity::new(artifact.identity.clone())
        .map_err(|_| Denial::InvalidIdentity)?;
    if artifact.path.is_empty()
        || artifact.path.len() > 32768
        || artifact.path.chars().any(char::is_control)
        || artifact.generation == Some(0)
    {
        return Err(Denial::InvalidObservation);
    }
    if let Some(value) = &artifact.range {
        range(value)?;
    }
    if artifact.family != "unrecognized" && family(&artifact.family).is_none() {
        return Err(Denial::InvalidObservation);
    }
    match &artifact.outcome {
        Outcome::Intact => {}
        Outcome::Damaged {
            cause,
            damaged_range,
            field,
            blast_radius,
        } => {
            identity(cause)?;
            identity(blast_radius)?;
            if let Some(field) = field {
                identity(field)?;
            }
            if let Some(value) = damaged_range {
                range(value)?;
            }
        }
        Outcome::Unsupported {
            axis,
            supported,
            range: value,
            ..
        } => {
            identity(axis)?;
            identity(supported)?;
            range(value)?;
        }
        Outcome::Unknown { reason } | Outcome::Indeterminate { reason } => identity(reason)?,
    }
    Ok(())
}

impl Outcome {
    pub(super) const fn posture(&self) -> PhysicalIntegrityPosture {
        match self {
            Self::Intact => PhysicalIntegrityPosture::Intact,
            Self::Damaged { .. } => PhysicalIntegrityPosture::Damaged,
            Self::Unsupported { .. } => PhysicalIntegrityPosture::Unsupported,
            Self::Unknown { .. } => PhysicalIntegrityPosture::Unknown,
            Self::Indeterminate { .. } => PhysicalIntegrityPosture::Indeterminate,
        }
    }
}

pub(super) fn family(value: &str) -> Option<worth_foundational::PhysicalArtifactFamily> {
    use worth_foundational::PhysicalArtifactFamily::*;
    Some(match value {
        "namespace_identity" => NamespaceIdentity,
        "physical_work_obligation" => PhysicalWorkObligation,
        "bootstrap_catalog" => BootstrapCatalog,
        "current_root_selector" => CurrentRootSelector,
        "previous_root_selector" => PreviousRootSelector,
        "root_manifest" => RootManifest,
        "root_routing_block" => RootRoutingBlock,
        "segment_membership_block" => SegmentMembershipBlock,
        "page_frame" => PageFrame,
        "extent_manifest" => ExtentManifest,
        "extent_chunk_frame" => ExtentChunkFrame,
        "free_space_header" => FreeSpaceHeader,
        "free_space_membership_block" => FreeSpaceMembershipBlock,
        "wal_frame" => WalFrame,
        "checkpoint_stream_header" => CheckpointStreamHeader,
        "checkpoint_dirty_basis" => CheckpointDirtyBasis,
        "checkpoint_binding_compaction" => CheckpointBindingCompaction,
        "checkpoint_binding" => CheckpointBinding,
        "checkpoint_footer" => CheckpointFooter,
        _ => return None,
    })
}
