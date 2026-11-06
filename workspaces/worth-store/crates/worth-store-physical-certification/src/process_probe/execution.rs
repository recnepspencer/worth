use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, ExitStatus};
use worth_foundational::CanonicalDigestId;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{
    artifact::output_artifact_identity, identity::ObservedProcessIdentity, wire_encoding,
    ProcessIdentityEvidence, ProcessProbeDeclaration, ProcessTerminationRequirement,
    SealedProcessProbeInput,
};
use crate::certification_child_process::publish_new_synced;

pub const PROCESS_PROBE_EVIDENCE_ROOT_ENV: &str = "WORTH_STORE_PROCESS_PROBE_EVIDENCE_ROOT";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessTermination {
    GracefulExit { code: Option<i32> },
    PanicUnwind { code: Option<i32> },
    Abort { platform_status: String },
    ParentKill { platform_status: String },
    OsTermination { platform_status: String },
}

#[derive(Debug)]
pub(crate) struct ObservedProcessTermination(ProcessTermination);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcessProbeExecution {
    pub schema_version: u32,
    pub declaration: ProcessProbeDeclaration,
    pub process: ProcessIdentityEvidence,
    pub termination: ProcessTermination,
    pub output_artifact_identity: CanonicalDigestId,
    pub evidence_identity: CanonicalDigestId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessProbeEvidenceDenial {
    UnadmittedEnvironment,
    InvalidChildObservation,
    MissingChildObservation,
    RoleMismatch,
    ProcessRelationshipMismatch,
    ExecutableMismatch,
    InputIdentityMismatch,
    WorkingDirectoryMismatch,
    EnvironmentMismatch,
    TerminationMismatch,
    OutputArtifactUnavailable,
    OutputArtifactMismatch,
    EvidenceWrite,
}

impl ProcessProbeExecution {
    pub(crate) fn observed(
        declaration: ProcessProbeDeclaration,
        input: &SealedProcessProbeInput,
        process: ObservedProcessIdentity,
        termination: ObservedProcessTermination,
        output_artifact: &Path,
    ) -> Result<Self, ProcessProbeEvidenceDenial> {
        if !termination_satisfies(declaration.required_termination(), &termination.0) {
            return Err(ProcessProbeEvidenceDenial::TerminationMismatch);
        }
        if declaration.input_identity() != input.identity() {
            return Err(ProcessProbeEvidenceDenial::InputIdentityMismatch);
        }
        if !input
            .admits_output_path(output_artifact)
            .map_err(|_| ProcessProbeEvidenceDenial::OutputArtifactMismatch)?
        {
            return Err(ProcessProbeEvidenceDenial::OutputArtifactMismatch);
        }
        let process = process.into_evidence();
        let output_artifact_identity = output_artifact_identity(output_artifact)
            .map_err(|_| ProcessProbeEvidenceDenial::OutputArtifactUnavailable)?;
        let mut evidence = Self {
            schema_version: 1,
            declaration,
            process,
            termination: termination.0,
            output_artifact_identity: CanonicalDigestId::new(output_artifact_identity),
            evidence_identity: CanonicalDigestId::new([0; 32]),
        };
        evidence.evidence_identity = wire_encoding::encode(&evidence)
            .map(|bytes| CanonicalDigestId::new(Sha256::digest(bytes).into()))
            .map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
        Ok(evidence)
    }
}

pub(crate) fn persist_execution(
    fallback_root: &Path,
    execution: &ProcessProbeExecution,
) -> Result<PathBuf, ProcessProbeEvidenceDenial> {
    let root = execution_root(fallback_root)?;
    let path = root.join(format!("{}.bin", hex(execution.evidence_identity.bytes())));
    let bytes =
        wire_encoding::encode(execution).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    match publish_new_synced(&path, &bytes) {
        Ok(()) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if fs::read(&path).ok().as_deref() == Some(bytes.as_slice()) {
                Ok(path)
            } else {
                Err(ProcessProbeEvidenceDenial::EvidenceWrite)
            }
        }
        Err(_) => Err(ProcessProbeEvidenceDenial::EvidenceWrite),
    }
}

fn execution_root(fallback_root: &Path) -> Result<PathBuf, ProcessProbeEvidenceDenial> {
    let Some(declared) = std::env::var_os(PROCESS_PROBE_EVIDENCE_ROOT_ENV).map(PathBuf::from)
    else {
        let root = fallback_root.join("process-probes");
        fs::create_dir_all(&root).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
        return Ok(root);
    };
    let workspace = std::env::current_dir()
        .and_then(fs::canonicalize)
        .map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    let admitted = workspace.join(".store-proof/evidence");
    fs::create_dir_all(&admitted).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    let admitted =
        fs::canonicalize(admitted).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    let declared = if declared.is_absolute() {
        declared
    } else {
        workspace.join(declared)
    };
    admit_declared_evidence_root(&admitted, &declared)?;
    fs::create_dir_all(&declared).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    let observed =
        fs::canonicalize(&declared).map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    if !observed.starts_with(&admitted) {
        return Err(ProcessProbeEvidenceDenial::EvidenceWrite);
    }
    Ok(declared)
}

pub(crate) fn admit_declared_evidence_root(
    admitted: &Path,
    declared: &Path,
) -> Result<(), ProcessProbeEvidenceDenial> {
    if declared
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(ProcessProbeEvidenceDenial::EvidenceWrite);
    }
    let mut existing = declared;
    while !existing.exists() {
        existing = existing
            .parent()
            .ok_or(ProcessProbeEvidenceDenial::EvidenceWrite)?;
    }
    let existing = existing
        .canonicalize()
        .map_err(|_| ProcessProbeEvidenceDenial::EvidenceWrite)?;
    if !existing.starts_with(admitted) {
        return Err(ProcessProbeEvidenceDenial::EvidenceWrite);
    }
    Ok(())
}

pub(crate) fn observe_graceful_exit(
    status: ExitStatus,
) -> Result<ObservedProcessTermination, ProcessProbeEvidenceDenial> {
    observe_required_exit(status, ProcessTerminationRequirement::GracefulExit)
}

pub(crate) fn observe_required_exit(
    status: ExitStatus,
    required: ProcessTerminationRequirement,
) -> Result<ObservedProcessTermination, ProcessProbeEvidenceDenial> {
    let platform_status = format!("{status:?}");
    let observed = if status.success() {
        ProcessTermination::GracefulExit {
            code: status.code(),
        }
    } else if status.code() == Some(101) {
        ProcessTermination::PanicUnwind {
            code: status.code(),
        }
    } else if is_abort_status(&status) {
        ProcessTermination::Abort { platform_status }
    } else {
        ProcessTermination::OsTermination { platform_status }
    };
    termination_satisfies(required, &observed)
        .then_some(ObservedProcessTermination(observed))
        .ok_or(ProcessProbeEvidenceDenial::TerminationMismatch)
}

pub(crate) fn terminate_by_parent(
    child: &mut Child,
) -> Result<ObservedProcessTermination, ProcessProbeEvidenceDenial> {
    child
        .kill()
        .map_err(|_| ProcessProbeEvidenceDenial::TerminationMismatch)?;
    let status = child
        .wait()
        .map_err(|_| ProcessProbeEvidenceDenial::TerminationMismatch)?;
    Ok(ObservedProcessTermination(ProcessTermination::ParentKill {
        platform_status: format!("{status:?}"),
    }))
}

pub(super) fn termination_satisfies(
    required: ProcessTerminationRequirement,
    observed: &ProcessTermination,
) -> bool {
    matches!(
        (required, observed),
        (
            ProcessTerminationRequirement::GracefulExit,
            ProcessTermination::GracefulExit { code: Some(0) }
        ) | (
            ProcessTerminationRequirement::PanicUnwind,
            ProcessTermination::PanicUnwind { .. }
        ) | (
            ProcessTerminationRequirement::Abort,
            ProcessTermination::Abort { .. }
        ) | (
            ProcessTerminationRequirement::ParentKill,
            ProcessTermination::ParentKill { .. }
        ) | (
            ProcessTerminationRequirement::OsTermination,
            ProcessTermination::OsTermination { .. }
        )
    )
}

fn hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(unix)]
fn is_abort_status(status: &ExitStatus) -> bool {
    use std::os::unix::process::ExitStatusExt;

    status.signal() == Some(6)
}

#[cfg(windows)]
fn is_abort_status(status: &ExitStatus) -> bool {
    matches!(status.code(), Some(3 | -1_073_740_791))
}

#[cfg(not(any(unix, windows)))]
fn is_abort_status(_status: &ExitStatus) -> bool {
    false
}
