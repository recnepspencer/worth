use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use worth_store_physical_backend::{OfflineMediaReadDenial, ReadOnlyOfflineMediaCapability};

use super::{BootstrapSourceArtifact, BootstrapSourceArtifactFamily};

#[derive(Debug)]
pub enum BootstrapSourceResolutionDenial {
    InvalidIdentity,
    InvalidFrontier,
    InvalidArtifact,
    DuplicateArtifact,
    MissingRequiredFamily(BootstrapSourceArtifactFamily),
    UnprovenContentClosure,
    ArtifactSetMismatch,
    ArtifactEscapesRoot,
    SymbolicLinkArtifact,
    ArtifactLengthMismatch,
    ArtifactDigestMismatch,
    InvalidBufferBudget,
    AllocationFailed,
    CounterOverflow,
    RootUnavailable,
    Media(OfflineMediaReadDenial),
}

impl From<OfflineMediaReadDenial> for BootstrapSourceResolutionDenial {
    fn from(value: OfflineMediaReadDenial) -> Self {
        Self::Media(value)
    }
}

#[derive(Debug)]
pub struct BootstrapSourceResolutionRequest {
    operation_identity: [u8; 32],
    evidence: BootstrapSourceEvidenceBinding,
    source_root: PathBuf,
    frontier: BootstrapSourceFrontier,
    artifacts: Vec<BootstrapSourceArtifact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapSourceEvidenceBinding {
    source_identity: [u8; 32],
    verification_identity: [u8; 32],
    source_lineage_identity: [u8; 32],
}

impl BootstrapSourceEvidenceBinding {
    pub fn from_independent_verification(
        source_identity: [u8; 32],
        verification_identity: [u8; 32],
        source_lineage_identity: [u8; 32],
    ) -> Result<Self, BootstrapSourceResolutionDenial> {
        if source_identity == [0; 32]
            || verification_identity == [0; 32]
            || source_lineage_identity == [0; 32]
        {
            return Err(BootstrapSourceResolutionDenial::InvalidIdentity);
        }
        Ok(Self {
            source_identity,
            verification_identity,
            source_lineage_identity,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapSourceFrontier {
    observed_lsn: u64,
    durable_lsn: u64,
    client_acknowledged_lsn: u64,
    replication_acknowledged_lsn: u64,
    authority_epoch: u64,
}

impl BootstrapSourceFrontier {
    pub const fn admit(
        observed_lsn: u64,
        durable_lsn: u64,
        client_acknowledged_lsn: u64,
        replication_acknowledged_lsn: u64,
        authority_epoch: u64,
    ) -> Result<Self, BootstrapSourceResolutionDenial> {
        if authority_epoch == 0 {
            return Err(BootstrapSourceResolutionDenial::InvalidIdentity);
        }
        if replication_acknowledged_lsn > client_acknowledged_lsn
            || client_acknowledged_lsn > durable_lsn
            || durable_lsn > observed_lsn
        {
            return Err(BootstrapSourceResolutionDenial::InvalidFrontier);
        }
        Ok(Self {
            observed_lsn,
            durable_lsn,
            client_acknowledged_lsn,
            replication_acknowledged_lsn,
            authority_epoch,
        })
    }
}

impl BootstrapSourceResolutionRequest {
    pub fn from_independent_verification(
        operation_identity: [u8; 32],
        evidence: BootstrapSourceEvidenceBinding,
        source_root: impl Into<PathBuf>,
        frontier: BootstrapSourceFrontier,
        mut artifacts: Vec<BootstrapSourceArtifact>,
    ) -> Result<Self, BootstrapSourceResolutionDenial> {
        if operation_identity == [0; 32] {
            return Err(BootstrapSourceResolutionDenial::InvalidIdentity);
        }
        artifacts.sort_by(|left, right| left.relative_path().cmp(right.relative_path()));
        if artifacts
            .windows(2)
            .any(|pair| pair[0].relative_path() == pair[1].relative_path())
        {
            return Err(BootstrapSourceResolutionDenial::DuplicateArtifact);
        }
        for family in [
            BootstrapSourceArtifactFamily::Authority,
            BootstrapSourceArtifactFamily::Checkpoint,
            BootstrapSourceArtifactFamily::Wal,
            BootstrapSourceArtifactFamily::Blob,
        ] {
            if !artifacts.iter().any(|artifact| artifact.family() == family) {
                return Err(BootstrapSourceResolutionDenial::MissingRequiredFamily(
                    family,
                ));
            }
        }
        Ok(Self {
            operation_identity,
            evidence,
            source_root: source_root.into(),
            frontier,
            artifacts,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapSourceResolutionCounters {
    artifacts_reopened: u64,
    bytes_read: u64,
    resident_buffer_bytes: u64,
}

impl BootstrapSourceResolutionCounters {
    pub const fn artifacts_reopened(self) -> u64 {
        self.artifacts_reopened
    }

    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub const fn resident_buffer_bytes(self) -> u64 {
        self.resident_buffer_bytes
    }
}

#[derive(Debug)]
pub struct ResolvedBootstrapRecoverySourceCut {
    operation_identity: [u8; 32],
    source_identity: [u8; 32],
    verification_identity: [u8; 32],
    source_lineage_identity: [u8; 32],
    source_root: PathBuf,
    artifact_paths: Vec<PathBuf>,
    frontier_identity: [u8; 32],
    resolution_identity: [u8; 32],
    counters: BootstrapSourceResolutionCounters,
}

impl ResolvedBootstrapRecoverySourceCut {
    pub const fn operation_identity(&self) -> [u8; 32] {
        self.operation_identity
    }

    pub const fn source_identity(&self) -> [u8; 32] {
        self.source_identity
    }

    pub const fn verification_identity(&self) -> [u8; 32] {
        self.verification_identity
    }

    pub const fn source_lineage_identity(&self) -> [u8; 32] {
        self.source_lineage_identity
    }

    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    pub fn artifact_paths(&self) -> &[PathBuf] {
        &self.artifact_paths
    }

    pub const fn frontier_identity(&self) -> [u8; 32] {
        self.frontier_identity
    }

    pub const fn resolution_identity(&self) -> [u8; 32] {
        self.resolution_identity
    }

    pub const fn counters(&self) -> BootstrapSourceResolutionCounters {
        self.counters
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalIsolationBootstrapSourceOwner;

impl PhysicalIsolationBootstrapSourceOwner {
    pub fn resolve(
        request: BootstrapSourceResolutionRequest,
        mut media: ReadOnlyOfflineMediaCapability,
        resident_buffer_bytes: usize,
    ) -> Result<ResolvedBootstrapRecoverySourceCut, BootstrapSourceResolutionDenial> {
        if resident_buffer_bytes == 0 {
            return Err(BootstrapSourceResolutionDenial::InvalidBufferBudget);
        }
        if !media.basis().is_content_addressed_closure() {
            return Err(BootstrapSourceResolutionDenial::UnprovenContentClosure);
        }
        if media.file_count() != request.artifacts.len() {
            return Err(BootstrapSourceResolutionDenial::ArtifactSetMismatch);
        }
        let source_root = std::fs::canonicalize(&request.source_root)
            .map_err(|_| BootstrapSourceResolutionDenial::RootUnavailable)?;
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(resident_buffer_bytes)
            .map_err(|_| BootstrapSourceResolutionDenial::AllocationFailed)?;
        buffer.resize(resident_buffer_bytes, 0);
        let mut bytes_read = 0_u64;
        for artifact in &request.artifacts {
            bytes_read = bytes_read
                .checked_add(verify_artifact(
                    &source_root,
                    artifact,
                    &mut media,
                    &mut buffer,
                )?)
                .ok_or(BootstrapSourceResolutionDenial::CounterOverflow)?;
        }
        media.revalidate_consistency()?;
        let counters = BootstrapSourceResolutionCounters {
            artifacts_reopened: u64::try_from(request.artifacts.len())
                .map_err(|_| BootstrapSourceResolutionDenial::CounterOverflow)?,
            bytes_read,
            resident_buffer_bytes: u64::try_from(resident_buffer_bytes)
                .map_err(|_| BootstrapSourceResolutionDenial::CounterOverflow)?,
        };
        Ok(resolve_cut(request, source_root, counters))
    }
}

fn verify_artifact(
    root: &Path,
    artifact: &BootstrapSourceArtifact,
    media: &mut ReadOnlyOfflineMediaCapability,
    buffer: &mut [u8],
) -> Result<u64, BootstrapSourceResolutionDenial> {
    let declared = root.join(artifact.relative_path());
    let metadata = std::fs::symlink_metadata(&declared)
        .map_err(|_| BootstrapSourceResolutionDenial::ArtifactSetMismatch)?;
    if metadata.file_type().is_symlink() {
        return Err(BootstrapSourceResolutionDenial::SymbolicLinkArtifact);
    }
    let path = std::fs::canonicalize(&declared)
        .map_err(|_| BootstrapSourceResolutionDenial::ArtifactSetMismatch)?;
    if !path.starts_with(root) {
        return Err(BootstrapSourceResolutionDenial::ArtifactEscapesRoot);
    }
    let index = media
        .file_index(&path)
        .ok_or(BootstrapSourceResolutionDenial::ArtifactSetMismatch)?;
    let length = media
        .file(index)
        .ok_or(BootstrapSourceResolutionDenial::ArtifactSetMismatch)?
        .length();
    if length != artifact.byte_length() {
        return Err(BootstrapSourceResolutionDenial::ArtifactLengthMismatch);
    }
    let mut digest = Sha256::new();
    let mut offset = 0_u64;
    while offset < length {
        let observation = media.read_bounded_into(index, offset, buffer)?;
        let read = observation.bytes_read();
        digest.update(&buffer[..read]);
        offset = offset
            .checked_add(read as u64)
            .ok_or(BootstrapSourceResolutionDenial::CounterOverflow)?;
    }
    let actual: [u8; 32] = digest.finalize().into();
    if actual != artifact.content_digest() {
        return Err(BootstrapSourceResolutionDenial::ArtifactDigestMismatch);
    }
    Ok(offset)
}

fn resolve_cut(
    request: BootstrapSourceResolutionRequest,
    source_root: PathBuf,
    counters: BootstrapSourceResolutionCounters,
) -> ResolvedBootstrapRecoverySourceCut {
    let frontier_identity = frontier_identity(&request);
    let resolution_identity = resolution_identity(&request, frontier_identity);
    let artifact_paths = request
        .artifacts
        .iter()
        .map(|artifact| artifact.relative_path().to_path_buf())
        .collect();
    ResolvedBootstrapRecoverySourceCut {
        operation_identity: request.operation_identity,
        source_identity: request.evidence.source_identity,
        verification_identity: request.evidence.verification_identity,
        source_lineage_identity: request.evidence.source_lineage_identity,
        source_root,
        artifact_paths,
        frontier_identity,
        resolution_identity,
        counters,
    }
}

fn frontier_identity(request: &BootstrapSourceResolutionRequest) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-bootstrap-source-frontier-v1");
    digest.update(request.frontier.observed_lsn.to_be_bytes());
    digest.update(request.frontier.durable_lsn.to_be_bytes());
    digest.update(request.frontier.client_acknowledged_lsn.to_be_bytes());
    digest.update(request.frontier.replication_acknowledged_lsn.to_be_bytes());
    digest.update(request.frontier.authority_epoch.to_be_bytes());
    digest.finalize().into()
}

fn resolution_identity(
    request: &BootstrapSourceResolutionRequest,
    frontier_identity: [u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-resolved-bootstrap-source-cut-v1");
    digest.update(request.operation_identity);
    digest.update(request.evidence.source_identity);
    digest.update(request.evidence.verification_identity);
    digest.update(request.evidence.source_lineage_identity);
    digest.update(frontier_identity);
    for artifact in &request.artifacts {
        digest.update([artifact.family().tag()]);
        update_path_digest(&mut digest, artifact.relative_path());
        digest.update(artifact.byte_length().to_be_bytes());
        digest.update(artifact.content_digest());
    }
    digest.finalize().into()
}

#[cfg(windows)]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    for unit in path.as_os_str().encode_wide() {
        digest.update(unit.to_le_bytes());
    }
}

#[cfg(unix)]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    digest.update(path.as_os_str().as_bytes());
}

#[cfg(not(any(windows, unix)))]
fn update_path_digest(digest: &mut Sha256, path: &Path) {
    digest.update(path.as_os_str().to_string_lossy().as_bytes());
}
