use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

mod bootstrap_lease;
mod bootstrap_source;
mod directory_durability;
mod lease_release;
mod record;
mod request;
mod source_artifact_path;
#[cfg(test)]
mod tests;

pub use bootstrap_lease::{BootstrapReachabilityLease, ResolvedBootstrapSourceCut};
pub use bootstrap_source::{
    BootstrapSourceArtifact, BootstrapSourceArtifactFamily, BootstrapSourceEvidenceBinding,
    BootstrapSourceFrontier, BootstrapSourceResolutionCounters, BootstrapSourceResolutionDenial,
    BootstrapSourceResolutionRequest, PhysicalIsolationBootstrapSourceOwner,
    ResolvedBootstrapRecoverySourceCut,
};
use directory_durability::sync_directory;
use lease_release::release_lease;
pub use lease_release::RecoverySourceLeaseReleaseReceipt;
pub use request::RecoverySourceLeaseRequest;
use source_artifact_path::{record_name, validate_source_artifact};

#[derive(Debug)]
pub enum RecoverySourceLeaseDenial {
    InvalidIdentity,
    EmptyClosure,
    InvalidArtifactName,
    DuplicateArtifact,
    MissingSourceArtifact { output_name: String },
    LeaseConflict,
    Io(std::io::Error),
    AllocationFailed,
    RecordTooLarge,
}

impl From<std::io::Error> for RecoverySourceLeaseDenial {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RecoverySourceReachabilityLease {
    identity: [u8; 32],
    operation_identity: [u8; 32],
    source_identity: [u8; 32],
    source_evidence_identity: [u8; 32],
    source_root: PathBuf,
    artifact_names: Vec<String>,
    durable_record: PathBuf,
}

impl RecoverySourceReachabilityLease {
    fn binding_fingerprint(&self) -> [u8; 32] {
        self.identity
    }
    const fn operation_identity(&self) -> [u8; 32] {
        self.operation_identity
    }
    const fn source_identity(&self) -> [u8; 32] {
        self.source_identity
    }
    const fn source_evidence_identity(&self) -> [u8; 32] {
        self.source_evidence_identity
    }
    fn source_root(&self) -> &Path {
        &self.source_root
    }
    fn artifact_names(&self) -> &[String] {
        &self.artifact_names
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedPitrSourceCut(RecoverySourceReachabilityLease);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PitrReachabilityLease(RecoverySourceReachabilityLease);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedRollbackSourceCut(RecoverySourceReachabilityLease);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackReachabilityLease(RecoverySourceReachabilityLease);

macro_rules! lease_accessors {
    ($type:ty) => {
        impl $type {
            pub fn source_root(&self) -> &Path {
                self.0.source_root()
            }
            pub const fn source_identity(&self) -> [u8; 32] {
                self.0.source_identity()
            }
            pub const fn source_evidence_identity(&self) -> [u8; 32] {
                self.0.source_evidence_identity()
            }
            pub fn artifact_names(&self) -> &[String] {
                self.0.artifact_names()
            }
            pub fn binding_fingerprint(&self) -> [u8; 32] {
                self.0.binding_fingerprint()
            }
            pub const fn operation_identity(&self) -> [u8; 32] {
                self.0.operation_identity()
            }
        }
    };
}

lease_accessors!(AdmittedPitrSourceCut);
lease_accessors!(PitrReachabilityLease);
lease_accessors!(AdmittedRollbackSourceCut);
lease_accessors!(RollbackReachabilityLease);

impl AdmittedPitrSourceCut {
    pub fn lease(self) -> PitrReachabilityLease {
        PitrReachabilityLease(self.0)
    }
}

impl AdmittedRollbackSourceCut {
    pub fn lease(self) -> RollbackReachabilityLease {
        RollbackReachabilityLease(self.0)
    }
}

impl PitrReachabilityLease {
    pub fn release(self) -> Result<RecoverySourceLeaseReleaseReceipt, RecoverySourceLeaseDenial> {
        release_lease(self.0)
    }
}

impl RollbackReachabilityLease {
    pub fn release(self) -> Result<RecoverySourceLeaseReleaseReceipt, RecoverySourceLeaseDenial> {
        release_lease(self.0)
    }
}

#[derive(Debug)]
pub struct RecoverySourceLeaseRegistry {
    root: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySourceLeaseKind {
    PointInTimeRecovery,
    Rollback,
    ReplicaBootstrap,
}

impl RecoverySourceLeaseKind {
    const fn tag(self) -> u8 {
        match self {
            Self::PointInTimeRecovery => 1,
            Self::Rollback => 2,
            Self::ReplicaBootstrap => 3,
        }
    }

    fn from_tag(tag: u8) -> Result<Self, RecoverySourceLeaseDenial> {
        match tag {
            1 => Ok(Self::PointInTimeRecovery),
            2 => Ok(Self::Rollback),
            3 => Ok(Self::ReplicaBootstrap),
            _ => Err(RecoverySourceLeaseDenial::LeaseConflict),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveredRecoverySourceLease {
    PointInTimeRecovery(PitrReachabilityLease),
    Rollback(RollbackReachabilityLease),
    ReplicaBootstrap(BootstrapReachabilityLease),
}

impl RecoverySourceLeaseRegistry {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, RecoverySourceLeaseDenial> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        sync_directory(&root)?;
        Ok(Self { root })
    }

    pub fn admit_pitr_source_cut(
        &self,
        request: RecoverySourceLeaseRequest,
    ) -> Result<AdmittedPitrSourceCut, RecoverySourceLeaseDenial> {
        Ok(AdmittedPitrSourceCut(self.admit(
            request,
            RecoverySourceLeaseKind::PointInTimeRecovery,
        )?))
    }

    pub fn admit_rollback_source_cut(
        &self,
        request: RecoverySourceLeaseRequest,
    ) -> Result<AdmittedRollbackSourceCut, RecoverySourceLeaseDenial> {
        Ok(AdmittedRollbackSourceCut(
            self.admit(request, RecoverySourceLeaseKind::Rollback)?,
        ))
    }

    pub fn admit_bootstrap_source_cut(
        &self,
        resolved: ResolvedBootstrapRecoverySourceCut,
    ) -> Result<ResolvedBootstrapSourceCut, RecoverySourceLeaseDenial> {
        let artifact_names = resolved
            .artifact_paths()
            .iter()
            .map(|path| record_name(path))
            .collect::<Result<Vec<_>, _>>()?;
        let request = RecoverySourceLeaseRequest::with_evidence_identity(
            resolved.operation_identity(),
            resolved.source_identity(),
            resolved.resolution_identity(),
            resolved.source_root(),
            artifact_names,
        );
        Ok(ResolvedBootstrapSourceCut(self.admit(
            request,
            RecoverySourceLeaseKind::ReplicaBootstrap,
        )?))
    }

    fn admit(
        &self,
        mut request: RecoverySourceLeaseRequest,
        operation_kind: RecoverySourceLeaseKind,
    ) -> Result<RecoverySourceReachabilityLease, RecoverySourceLeaseDenial> {
        if request.operation_identity == [0; 32]
            || request.source_identity == [0; 32]
            || request.source_evidence_identity == [0; 32]
        {
            return Err(RecoverySourceLeaseDenial::InvalidIdentity);
        }
        if request.artifact_names.is_empty() {
            return Err(RecoverySourceLeaseDenial::EmptyClosure);
        }
        request.artifact_names.sort();
        if request
            .artifact_names
            .windows(2)
            .any(|names| names[0] == names[1])
        {
            return Err(RecoverySourceLeaseDenial::DuplicateArtifact);
        }
        let source_root = std::fs::canonicalize(&request.source_root)?;
        for name in &request.artifact_names {
            validate_source_artifact(&source_root, name)?;
        }
        let content = record::encode(
            operation_kind,
            request.operation_identity,
            request.source_identity,
            request.source_evidence_identity,
            &source_root,
            &request.artifact_names,
        )?;
        let identity: [u8; 32] = Sha256::digest(&content).into();
        let durable_record = self.root.join(record::filename(identity));
        record::persist(&self.root, &durable_record, &content)?;
        Ok(RecoverySourceReachabilityLease {
            identity,
            operation_identity: request.operation_identity,
            source_identity: request.source_identity,
            source_evidence_identity: request.source_evidence_identity,
            source_root,
            artifact_names: request.artifact_names,
            durable_record,
        })
    }

    pub fn recover_active(
        &self,
    ) -> Result<Vec<RecoveredRecoverySourceLease>, RecoverySourceLeaseDenial> {
        let mut records = std::fs::read_dir(&self.root)?.collect::<Result<Vec<_>, _>>()?;
        records.sort_by_key(std::fs::DirEntry::file_name);
        let mut recovered = Vec::new();
        recovered
            .try_reserve(records.len())
            .map_err(|_| RecoverySourceLeaseDenial::AllocationFailed)?;
        for entry in records {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let Some(identity) = record::identity_from_filename(name) else {
                continue;
            };
            let content = std::fs::read(entry.path())?;
            let persisted = record::decode(&content, identity)?;
            validate_recovered_closure(&persisted.source_root, &persisted.artifact_names)?;
            let lease = RecoverySourceReachabilityLease {
                identity,
                operation_identity: persisted.operation_identity,
                source_identity: persisted.source_identity,
                source_evidence_identity: persisted.source_evidence_identity,
                source_root: persisted.source_root,
                artifact_names: persisted.artifact_names,
                durable_record: entry.path(),
            };
            recovered.push(match persisted.kind {
                RecoverySourceLeaseKind::PointInTimeRecovery => {
                    RecoveredRecoverySourceLease::PointInTimeRecovery(PitrReachabilityLease(lease))
                }
                RecoverySourceLeaseKind::Rollback => {
                    RecoveredRecoverySourceLease::Rollback(RollbackReachabilityLease(lease))
                }
                RecoverySourceLeaseKind::ReplicaBootstrap => {
                    RecoveredRecoverySourceLease::ReplicaBootstrap(BootstrapReachabilityLease(
                        lease,
                    ))
                }
            });
        }
        Ok(recovered)
    }

    pub fn recover_bound(
        &self,
        operation_identity: [u8; 32],
        expected_kind: RecoverySourceLeaseKind,
    ) -> Result<RecoveredRecoverySourceLease, RecoverySourceLeaseDenial> {
        let mut matching = self
            .recover_active()?
            .into_iter()
            .filter(|lease| match lease {
                RecoveredRecoverySourceLease::PointInTimeRecovery(lease) => {
                    expected_kind == RecoverySourceLeaseKind::PointInTimeRecovery
                        && lease.operation_identity() == operation_identity
                }
                RecoveredRecoverySourceLease::Rollback(lease) => {
                    expected_kind == RecoverySourceLeaseKind::Rollback
                        && lease.operation_identity() == operation_identity
                }
                RecoveredRecoverySourceLease::ReplicaBootstrap(lease) => {
                    expected_kind == RecoverySourceLeaseKind::ReplicaBootstrap
                        && lease.operation_identity() == operation_identity
                }
            });
        let lease = matching
            .next()
            .ok_or(RecoverySourceLeaseDenial::LeaseConflict)?;
        if matching.next().is_some() {
            return Err(RecoverySourceLeaseDenial::LeaseConflict);
        }
        Ok(lease)
    }
}

fn validate_recovered_closure(
    source_root: &Path,
    artifact_names: &[String],
) -> Result<(), RecoverySourceLeaseDenial> {
    if artifact_names.is_empty() {
        return Err(RecoverySourceLeaseDenial::EmptyClosure);
    }
    if artifact_names.windows(2).any(|names| names[0] >= names[1]) {
        return Err(RecoverySourceLeaseDenial::LeaseConflict);
    }
    for name in artifact_names {
        validate_source_artifact(source_root, name)?;
    }
    Ok(())
}
