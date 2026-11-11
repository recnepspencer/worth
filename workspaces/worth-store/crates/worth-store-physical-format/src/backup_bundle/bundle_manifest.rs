use std::path::{Path, PathBuf};
use worth_foundational::CanonicalDigestId;
mod artifact_manifest_row;
mod declaration;

pub use artifact_manifest_row::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactFormat,
    BackupBundleArtifactManifestRow,
};
pub use declaration::{
    BackupBundleManifestDeclaration, BackupBundleManifestIdentity, BackupBundleRecoveryCoordinates,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundleManifest {
    cut_identity: CanonicalDigestId,
    store_lineage: String,
    root_generation: u64,
    manifest_generation: u64,
    checkpoint_identity: String,
    durable_checkpoint_lsn: u64,
    wal_start_lsn: u64,
    wal_end_exclusive_lsn: u64,
    acknowledged_frontier: u64,
    security_scope_fingerprint: u64,
    artifacts: Vec<BackupBundleArtifactManifestRow>,
    artifact_closure_digest: CanonicalDigestId,
}

impl BackupBundleManifest {
    pub fn canonical(declaration: BackupBundleManifestDeclaration) -> Option<Self> {
        Self::canonical_checked(declaration).ok()
    }

    pub fn canonical_checked(
        declaration: BackupBundleManifestDeclaration,
    ) -> Result<Self, BackupBundleManifestConstructionDenial> {
        let BackupBundleManifestDeclaration {
            identity,
            recovery,
            security_scope_fingerprint,
            artifacts,
        } = declaration;
        let BackupBundleManifestIdentity {
            cut_identity,
            store_lineage,
            root_generation,
            manifest_generation,
        } = identity;
        let BackupBundleRecoveryCoordinates {
            checkpoint_identity,
            durable_checkpoint_lsn,
            wal_half_open_interval,
            acknowledged_frontier,
        } = recovery;
        if store_lineage.trim().is_empty()
            || checkpoint_identity.trim().is_empty()
            || root_generation == 0
            || manifest_generation == 0
            || wal_half_open_interval.0 > durable_checkpoint_lsn
            || durable_checkpoint_lsn > wal_half_open_interval.1
            || acknowledged_frontier < wal_half_open_interval.1
        {
            return Err(BackupBundleManifestConstructionDenial::InvalidManifest);
        }
        let artifacts = canonicalize_declared_artifacts(artifacts)?;
        Ok(Self {
            cut_identity,
            store_lineage,
            root_generation,
            manifest_generation,
            checkpoint_identity,
            durable_checkpoint_lsn,
            wal_start_lsn: wal_half_open_interval.0,
            wal_end_exclusive_lsn: wal_half_open_interval.1,
            acknowledged_frontier,
            security_scope_fingerprint,
            artifact_closure_digest: CanonicalDigestId::new(
                super::backup_canonical_artifact_closure_digest(&artifacts),
            ),
            artifacts,
        })
    }
    pub const fn cut_identity(&self) -> [u8; 32] {
        *self.cut_identity.bytes()
    }
    pub fn store_lineage(&self) -> &str {
        &self.store_lineage
    }
    pub const fn root_generation(&self) -> u64 {
        self.root_generation
    }
    pub const fn manifest_generation(&self) -> u64 {
        self.manifest_generation
    }
    pub fn checkpoint_identity(&self) -> &str {
        &self.checkpoint_identity
    }
    pub const fn durable_checkpoint_lsn(&self) -> u64 {
        self.durable_checkpoint_lsn
    }
    pub const fn wal_half_open_interval(&self) -> (u64, u64) {
        (self.wal_start_lsn, self.wal_end_exclusive_lsn)
    }
    pub const fn acknowledged_frontier(&self) -> u64 {
        self.acknowledged_frontier
    }
    pub const fn security_scope_fingerprint(&self) -> u64 {
        self.security_scope_fingerprint
    }
    pub fn artifacts(&self) -> &[BackupBundleArtifactManifestRow] {
        &self.artifacts
    }
    pub const fn artifact_closure_digest(&self) -> [u8; 32] {
        *self.artifact_closure_digest.bytes()
    }

    pub(crate) fn from_decoded_parts(
        declaration: BackupBundleManifestDeclaration,
        artifact_closure_digest: [u8; 32],
    ) -> Self {
        let BackupBundleManifestDeclaration {
            identity,
            recovery,
            security_scope_fingerprint,
            artifacts,
        } = declaration;
        Self {
            cut_identity: identity.cut_identity,
            store_lineage: identity.store_lineage,
            root_generation: identity.root_generation,
            manifest_generation: identity.manifest_generation,
            checkpoint_identity: recovery.checkpoint_identity,
            durable_checkpoint_lsn: recovery.durable_checkpoint_lsn,
            wal_start_lsn: recovery.wal_half_open_interval.0,
            wal_end_exclusive_lsn: recovery.wal_half_open_interval.1,
            acknowledged_frontier: recovery.acknowledged_frontier,
            security_scope_fingerprint,
            artifacts,
            artifact_closure_digest: CanonicalDigestId::new(artifact_closure_digest),
        }
    }

    pub(crate) fn owned_allocation_bytes(&self) -> Option<u64> {
        let artifact_storage = self
            .artifacts
            .capacity()
            .checked_mul(std::mem::size_of::<BackupBundleArtifactManifestRow>())?;
        let fixed = u64::try_from(self.store_lineage.capacity())
            .ok()?
            .checked_add(u64::try_from(self.checkpoint_identity.capacity()).ok()?)?
            .checked_add(u64::try_from(artifact_storage).ok()?)?;
        self.artifacts.iter().try_fold(fixed, |total, row| {
            total.checked_add(row.owned_allocation_bytes()?)
        })
    }

    pub(crate) fn admit_decoded(
        self,
        maximum_owned_allocation_bytes: u64,
    ) -> Result<(Self, u64), BackupBundleManifestAdmissionDenial> {
        if !self.has_canonical_valid_shape() {
            return Err(BackupBundleManifestAdmissionDenial::InvalidManifest);
        }
        let manifest_bytes = self
            .owned_allocation_bytes()
            .ok_or(BackupBundleManifestAdmissionDenial::AllocationCountOverflow)?;
        let requested_workspace_bytes = u64::try_from(self.artifacts.len())
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<&str>() as u64))
            .ok_or(BackupBundleManifestAdmissionDenial::AllocationCountOverflow)?;
        let requested_peak = manifest_bytes
            .checked_add(requested_workspace_bytes)
            .ok_or(BackupBundleManifestAdmissionDenial::AllocationCountOverflow)?;
        if requested_peak > maximum_owned_allocation_bytes {
            return Err(
                BackupBundleManifestAdmissionDenial::AllocationLimitExceeded {
                    observed_bytes: requested_peak,
                    maximum_bytes: maximum_owned_allocation_bytes,
                },
            );
        }
        let (unique_output_names, workspace_bytes) = output_name_index(&self.artifacts)
            .map_err(|_| BackupBundleManifestAdmissionDenial::AllocationFailed)?;
        if !unique_output_names {
            return Err(BackupBundleManifestAdmissionDenial::InvalidManifest);
        }
        let peak_owned_allocation_bytes = manifest_bytes
            .checked_add(workspace_bytes)
            .ok_or(BackupBundleManifestAdmissionDenial::AllocationCountOverflow)?;
        if peak_owned_allocation_bytes > maximum_owned_allocation_bytes {
            return Err(
                BackupBundleManifestAdmissionDenial::AllocationLimitExceeded {
                    observed_bytes: peak_owned_allocation_bytes,
                    maximum_bytes: maximum_owned_allocation_bytes,
                },
            );
        }
        Ok((self, peak_owned_allocation_bytes))
    }

    fn has_canonical_valid_shape(&self) -> bool {
        !self.store_lineage.trim().is_empty()
            && !self.checkpoint_identity.trim().is_empty()
            && self.root_generation > 0
            && self.manifest_generation > 0
            && !self.artifacts.is_empty()
            && self.wal_start_lsn <= self.durable_checkpoint_lsn
            && self.durable_checkpoint_lsn <= self.wal_end_exclusive_lsn
            && self.acknowledged_frontier >= self.wal_end_exclusive_lsn
            && self
                .artifacts
                .iter()
                .all(BackupBundleArtifactManifestRow::is_valid)
            && self.artifacts.windows(2).all(|pair| {
                artifact_order(&pair[0], &pair[1]) != std::cmp::Ordering::Greater
                    && (pair[0].family() != pair[1].family()
                        || pair[0].identity() != pair[1].identity())
            })
            && super::backup_canonical_artifact_closure_digest(&self.artifacts)
                == *self.artifact_closure_digest.bytes()
    }
}

fn canonicalize_declared_artifacts(
    mut artifacts: Vec<BackupBundleArtifactManifestRow>,
) -> Result<Vec<BackupBundleArtifactManifestRow>, BackupBundleManifestConstructionDenial> {
    artifacts.sort_by(artifact_order);
    let unique_semantic_identities = artifacts.windows(2).all(|pair| {
        pair[0].family() != pair[1].family() || pair[0].identity() != pair[1].identity()
    });
    let unique_output_names = output_name_index(&artifacts)
        .map_err(|_| BackupBundleManifestConstructionDenial::AllocationFailed)?
        .0;
    let valid_rows = artifacts
        .iter()
        .all(BackupBundleArtifactManifestRow::is_valid);
    if artifacts.is_empty() || !unique_semantic_identities || !unique_output_names || !valid_rows {
        Err(BackupBundleManifestConstructionDenial::InvalidManifest)
    } else {
        Ok(artifacts)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupBundleManifestConstructionDenial {
    InvalidManifest,
    AllocationFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackupBundleManifestAdmissionDenial {
    InvalidManifest,
    AllocationFailed,
    AllocationCountOverflow,
    AllocationLimitExceeded {
        observed_bytes: u64,
        maximum_bytes: u64,
    },
}

fn artifact_order(
    left: &BackupBundleArtifactManifestRow,
    right: &BackupBundleArtifactManifestRow,
) -> std::cmp::Ordering {
    left.family()
        .cmp(&right.family())
        .then_with(|| left.identity().cmp(right.identity()))
        .then_with(|| left.output_name().cmp(right.output_name()))
}

fn output_name_index(
    artifacts: &[BackupBundleArtifactManifestRow],
) -> Result<(bool, u64), std::collections::TryReserveError> {
    let mut output_names = Vec::new();
    output_names.try_reserve_exact(artifacts.len())?;
    output_names.extend(artifacts.iter().map(|row| row.output_name()));
    output_names.sort_unstable();
    let unique = output_names.windows(2).all(|pair| pair[0] != pair[1]);
    let workspace_bytes = output_names
        .capacity()
        .checked_mul(std::mem::size_of::<&str>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .unwrap_or(u64::MAX);
    Ok((unique, workspace_bytes))
}

#[derive(Debug, PartialEq, Eq)]
pub struct MaterializedBackupBundle {
    root: PathBuf,
    manifest: BackupBundleManifest,
    manifest_digest: CanonicalDigestId,
    manifest_read: super::BackupBundleManifestReadObservation,
}

impl MaterializedBackupBundle {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub const fn manifest(&self) -> &BackupBundleManifest {
        &self.manifest
    }
    pub const fn manifest_digest(&self) -> [u8; 32] {
        *self.manifest_digest.bytes()
    }
    pub const fn manifest_read_observation(&self) -> super::BackupBundleManifestReadObservation {
        self.manifest_read
    }
    pub(crate) fn new(
        root: PathBuf,
        manifest: BackupBundleManifest,
        manifest_digest: [u8; 32],
        manifest_read: super::BackupBundleManifestReadObservation,
    ) -> Self {
        Self {
            root,
            manifest,
            manifest_digest: CanonicalDigestId::new(manifest_digest),
            manifest_read,
        }
    }
}

#[cfg(test)]
mod record_extent_owner_tests;
#[cfg(test)]
mod tests;
