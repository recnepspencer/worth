use std::path::PathBuf;
use worth_foundational::CanonicalDigestId;

use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_physical_format::{
    BackupBundleFormatAuthority, BackupBundleManifest, BackupBundleManifestDeclaration,
    BackupBundleManifestIdentity, BackupBundleRecoveryCoordinates,
};

use super::cut_manifest::portable_row;
use super::cut_recovery_codec::{
    decode_backup_cut_recovery, encode_backup_cut_recovery, BackupCutRecoveryCodecDenial,
};
use super::{
    AdmittedBackupCut, BackupCutAdmissionDenial, BackupCutManifestDenial,
    BackupReachabilityLeasePersistenceRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BackupCutRecoverySource {
    pub(super) output_name: String,
    pub(super) path: PathBuf,
    pub(super) physical_identity: CanonicalDigestId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupCutRecoveryRecord {
    pub(super) authority_identity: StoreCurrentAuthorityIdentity,
    pub(super) security_progression_fingerprint: u64,
    pub(super) format_profile: String,
    pub(super) backend_profile: String,
    pub(super) manifest: BackupBundleManifest,
    pub(super) sources: Vec<BackupCutRecoverySource>,
    encoded: Vec<u8>,
}

#[derive(Debug)]
pub enum BackupCutRecoveryDenial {
    InvalidEncoding,
    UnsupportedPathPlatform,
    AllocationFailed,
    SizeLimitExceeded,
    InvalidCutInvariant,
    Format(worth_store_physical_format::BackupBundleFormatDenial),
}

#[derive(Debug)]
pub enum BackupCutReadmissionDenial {
    AuthorityIdentityChanged,
    SecurityScopeAuthorityChanged,
    SecurityScopeChanged,
    SecurityProofProgressionChanged,
    MissingSource(String),
    SourceObservation {
        output_name: String,
        source: worth_store_physical_backend::PhysicalBackupArtifactObservationDenial,
    },
    SourceLengthChanged(String),
    SourceDigestChanged(String),
    SourcePhysicalIdentityChanged(String),
    ArtifactInvariant(String),
    Manifest(BackupCutManifestDenial),
    Coordinates,
    Admission(BackupCutAdmissionDenial),
    CutIdentityChanged,
    AllocationFailed,
}

impl AdmittedBackupCut {
    pub fn recovery_record(&self) -> Result<BackupCutRecoveryRecord, BackupCutRecoveryDenial> {
        BackupCutRecoveryRecord::from_admitted(self)
    }
}

impl BackupCutRecoveryRecord {
    fn from_admitted(cut: &AdmittedBackupCut) -> Result<Self, BackupCutRecoveryDenial> {
        let artifact_count = cut.manifest().artifacts().len();
        let mut rows = Vec::new();
        rows.try_reserve_exact(artifact_count)
            .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?;
        for (index, artifact) in cut.manifest().artifacts().iter().enumerate() {
            rows.push(
                portable_row(index, artifact)
                    .ok_or(BackupCutRecoveryDenial::InvalidCutInvariant)?,
            );
        }
        let mut sources = Vec::new();
        sources
            .try_reserve_exact(artifact_count)
            .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?;
        for (row, artifact) in rows.iter().zip(cut.manifest().artifacts()) {
            sources.push(BackupCutRecoverySource {
                output_name: copy_string(row.output_name())
                    .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?,
                path: copy_path(artifact.source_path())
                    .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?,
                physical_identity: CanonicalDigestId::new(artifact.physical_identity()),
            });
        }
        let coordinates = cut.coordinates();
        let security = cut.security_scope().receipt_id();
        let manifest = BackupBundleManifest::canonical_checked(
            BackupBundleManifestDeclaration::new(
                BackupBundleManifestIdentity {
                    cut_identity: worth_foundational::CanonicalDigestId::new(cut.identity()),
                    store_lineage: coordinates.store_lineage().to_owned(),
                    root_generation: coordinates.root_generation(),
                    manifest_generation: coordinates.manifest_generation(),
                },
                BackupBundleRecoveryCoordinates {
                    checkpoint_identity: coordinates.checkpoint_identity().to_owned(),
                    durable_checkpoint_lsn: coordinates.durable_checkpoint_lsn(),
                    wal_half_open_interval: coordinates.wal_half_open_interval(),
                    acknowledged_frontier: coordinates.acknowledged_frontier(),
                },
                security.security_scope_fingerprint(),
                rows,
            ),
        )
        .map_err(|denial| match denial {
            worth_store_physical_format::BackupBundleManifestConstructionDenial::InvalidManifest => {
                BackupCutRecoveryDenial::InvalidCutInvariant
            }
            worth_store_physical_format::BackupBundleManifestConstructionDenial::AllocationFailed => {
                BackupCutRecoveryDenial::AllocationFailed
            }
        })?;
        let format = BackupBundleFormatAuthority::canonical();
        let manifest_bytes = format
            .encode_manifest(&manifest)
            .map_err(BackupCutRecoveryDenial::Format)?;
        let encoded = encode_backup_cut_recovery(
            cut.authority_identity().fingerprint(),
            security.proof_progression_fingerprint(),
            coordinates.format_profile(),
            coordinates.backend_profile(),
            &manifest_bytes,
            &sources,
        )
        .map_err(map_codec_denial)?;
        Ok(Self {
            authority_identity: cut.authority_identity(),
            security_progression_fingerprint: security.proof_progression_fingerprint(),
            format_profile: copy_string(coordinates.format_profile())
                .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?,
            backend_profile: copy_string(coordinates.backend_profile())
                .map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?,
            manifest,
            sources,
            encoded,
        })
    }

    pub fn recover(encoded: &[u8]) -> Result<Self, BackupCutRecoveryDenial> {
        let decoded = decode_backup_cut_recovery(encoded).map_err(map_codec_denial)?;
        let manifest = BackupBundleFormatAuthority::canonical()
            .decode_manifest(&decoded.manifest_bytes)
            .map_err(BackupCutRecoveryDenial::Format)?;
        validate_source_closure(&manifest, &decoded.sources)?;
        Ok(Self {
            authority_identity: StoreCurrentAuthorityIdentity::from_persisted_fingerprint(
                *decoded.authority_fingerprint.bytes(),
            ),
            security_progression_fingerprint: decoded.security_progression_fingerprint,
            format_profile: decoded.format_profile,
            backend_profile: decoded.backend_profile,
            manifest,
            sources: decoded.sources,
            encoded: copy_bytes(encoded).map_err(|_| BackupCutRecoveryDenial::AllocationFailed)?,
        })
    }

    pub const fn cut_identity(&self) -> [u8; 32] {
        self.manifest.cut_identity()
    }

    pub const fn authority_identity(&self) -> StoreCurrentAuthorityIdentity {
        self.authority_identity
    }

    pub fn recovery_bytes(&self) -> &[u8] {
        &self.encoded
    }

    pub fn lease_persistence_record(
        &self,
    ) -> Result<BackupReachabilityLeasePersistenceRecord, BackupCutRecoveryDenial> {
        BackupReachabilityLeasePersistenceRecord::from_recovery_owners(
            self.cut_identity(),
            self.manifest
                .artifacts()
                .iter()
                .map(|row| row.reclaim_owner().generation_owner()),
        )
        .map_err(|_| BackupCutRecoveryDenial::InvalidCutInvariant)
    }
}

fn validate_source_closure(
    manifest: &BackupBundleManifest,
    sources: &[BackupCutRecoverySource],
) -> Result<(), BackupCutRecoveryDenial> {
    if manifest.artifacts().len() != sources.len()
        || manifest
            .artifacts()
            .iter()
            .zip(sources)
            .any(|(row, source)| row.output_name() != source.output_name)
    {
        return Err(BackupCutRecoveryDenial::InvalidCutInvariant);
    }
    Ok(())
}

const fn map_codec_denial(denial: BackupCutRecoveryCodecDenial) -> BackupCutRecoveryDenial {
    match denial {
        BackupCutRecoveryCodecDenial::InvalidEncoding => BackupCutRecoveryDenial::InvalidEncoding,
        BackupCutRecoveryCodecDenial::UnsupportedPathPlatform => {
            BackupCutRecoveryDenial::UnsupportedPathPlatform
        }
        BackupCutRecoveryCodecDenial::AllocationFailed => BackupCutRecoveryDenial::AllocationFailed,
        BackupCutRecoveryCodecDenial::SizeLimitExceeded => {
            BackupCutRecoveryDenial::SizeLimitExceeded
        }
    }
}

fn copy_string(value: &str) -> Result<String, std::collections::TryReserveError> {
    let mut copied = String::new();
    copied.try_reserve_exact(value.len())?;
    copied.push_str(value);
    Ok(copied)
}

fn copy_bytes(value: &[u8]) -> Result<Vec<u8>, std::collections::TryReserveError> {
    let mut copied = Vec::new();
    copied.try_reserve_exact(value.len())?;
    copied.extend_from_slice(value);
    Ok(copied)
}

fn copy_path(value: &std::path::Path) -> Result<PathBuf, std::collections::TryReserveError> {
    let mut copied = PathBuf::new();
    copied.try_reserve_exact(value.as_os_str().len())?;
    copied.push(value);
    Ok(copied)
}
