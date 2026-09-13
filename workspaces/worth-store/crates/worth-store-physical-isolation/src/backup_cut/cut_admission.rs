use sha2::{Digest, Sha256};
use worth_store_authority::{StoreCurrentAuthorityIdentity, StoreCurrentAuthorityWitness};
use worth_store_security::{
    StoreAuthorityBoundSecurityScopeReceipt, StoreSecurityScopeAdmissionReceipt,
};

use super::{
    BackupArtifactCoverage, BackupCutManifest, BackupCutStoragePosture, BackupReachabilityLease,
    BackupReachabilityLeaseRecoveryDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupCutCoordinates {
    store_lineage: String,
    root_generation: u64,
    manifest_generation: u64,
    checkpoint_identity: String,
    durable_checkpoint_lsn: u64,
    wal_start_lsn: u64,
    wal_end_exclusive_lsn: u64,
    acknowledged_frontier: u64,
    format_profile: String,
    backend_profile: String,
}

impl BackupCutCoordinates {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store_lineage: impl Into<String>,
        root_generation: u64,
        manifest_generation: u64,
        checkpoint_identity: impl Into<String>,
        durable_checkpoint_lsn: u64,
        wal_start_lsn: u64,
        wal_end_exclusive_lsn: u64,
        acknowledged_frontier: u64,
        format_profile: impl Into<String>,
        backend_profile: impl Into<String>,
    ) -> Option<Self> {
        let value = Self {
            store_lineage: store_lineage.into(),
            root_generation,
            manifest_generation,
            checkpoint_identity: checkpoint_identity.into(),
            durable_checkpoint_lsn,
            wal_start_lsn,
            wal_end_exclusive_lsn,
            acknowledged_frontier,
            format_profile: format_profile.into(),
            backend_profile: backend_profile.into(),
        };
        if value.store_lineage.trim().is_empty()
            || value.checkpoint_identity.trim().is_empty()
            || value.format_profile.trim().is_empty()
            || value.backend_profile.trim().is_empty()
            || root_generation == 0
            || manifest_generation == 0
            || wal_start_lsn > durable_checkpoint_lsn
            || durable_checkpoint_lsn > wal_end_exclusive_lsn
            || acknowledged_frontier < wal_end_exclusive_lsn
        {
            None
        } else {
            Some(value)
        }
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
    pub fn format_profile(&self) -> &str {
        &self.format_profile
    }
    pub fn backend_profile(&self) -> &str {
        &self.backend_profile
    }
}

#[derive(Debug)]
pub struct BackupCutAdmissionRequest {
    security_scope: StoreAuthorityBoundSecurityScopeReceipt,
    coordinates: BackupCutCoordinates,
    manifest: BackupCutManifest,
    storage_posture: BackupCutStoragePosture,
}

impl BackupCutAdmissionRequest {
    pub fn new(
        security_scope: StoreAuthorityBoundSecurityScopeReceipt,
        coordinates: BackupCutCoordinates,
        manifest: BackupCutManifest,
        storage_posture: BackupCutStoragePosture,
    ) -> Self {
        Self {
            security_scope,
            coordinates,
            manifest,
            storage_posture,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AdmittedBackupCut {
    identity: [u8; 32],
    authority_identity: StoreCurrentAuthorityIdentity,
    security_scope: StoreAuthorityBoundSecurityScopeReceipt,
    coordinates: BackupCutCoordinates,
    manifest: BackupCutManifest,
    lease: BackupReachabilityLease,
}

impl AdmittedBackupCut {
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub const fn authority_identity(&self) -> StoreCurrentAuthorityIdentity {
        self.authority_identity
    }
    pub const fn security_scope(&self) -> StoreSecurityScopeAdmissionReceipt {
        self.security_scope.receipt()
    }
    pub const fn authority_bound_security_scope(&self) -> StoreAuthorityBoundSecurityScopeReceipt {
        self.security_scope
    }
    pub const fn coordinates(&self) -> &BackupCutCoordinates {
        &self.coordinates
    }
    pub const fn manifest(&self) -> &BackupCutManifest {
        &self.manifest
    }
    pub const fn lease(&self) -> &BackupReachabilityLease {
        &self.lease
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupCutAdmissionDenial {
    SecurityScopeAuthorityMismatch,
    ReachabilityAuthorityMismatch,
    FormatProfileMismatch,
    BackendProfileMismatch,
    RootManifestCardinality,
    CheckpointManifestCardinality,
    RootGenerationMismatch,
    CheckpointIdentityMismatch,
    CheckpointGenerationMismatch,
    CheckpointFrontierMismatch,
    RootCoverageMismatch,
    CheckpointCoverageMismatch,
    WalCoverageGap,
    Lease(BackupReachabilityLeaseRecoveryDenial),
    AllocationFailed,
}

#[derive(Debug, Clone, Copy)]
pub struct BackupCutAdmissionAuthority<'a> {
    current_authority: &'a StoreCurrentAuthorityWitness,
}

impl<'a> BackupCutAdmissionAuthority<'a> {
    pub const fn for_current_store(current_authority: &'a StoreCurrentAuthorityWitness) -> Self {
        Self { current_authority }
    }
    pub fn admit(
        self,
        request: BackupCutAdmissionRequest,
    ) -> Result<AdmittedBackupCut, BackupCutAdmissionDenial> {
        let authority_identity = self.current_authority.authority_identity();
        if request.security_scope.authority_identity() != authority_identity {
            return Err(BackupCutAdmissionDenial::SecurityScopeAuthorityMismatch);
        }
        if request
            .manifest
            .source_authority_identity()
            .is_some_and(|source| source != authority_identity)
        {
            return Err(BackupCutAdmissionDenial::ReachabilityAuthorityMismatch);
        }
        if request.coordinates.format_profile() != request.storage_posture.format_profile() {
            return Err(BackupCutAdmissionDenial::FormatProfileMismatch);
        }
        if request.coordinates.backend_profile() != request.storage_posture.backend_profile() {
            return Err(BackupCutAdmissionDenial::BackendProfileMismatch);
        }
        validate_manifest_coordinates(&request.coordinates, &request.manifest, authority_identity)?;
        let identity = cut_identity(
            authority_identity,
            request.security_scope.receipt(),
            &request.coordinates,
            &request.manifest,
        );
        let lease = BackupReachabilityLease::for_admitted_cut(identity, &request.manifest)
            .map_err(BackupCutAdmissionDenial::Lease)?;
        Ok(AdmittedBackupCut {
            identity,
            authority_identity,
            security_scope: request.security_scope,
            coordinates: request.coordinates,
            manifest: request.manifest,
            lease,
        })
    }
}

fn validate_manifest_coordinates(
    coordinates: &BackupCutCoordinates,
    manifest: &BackupCutManifest,
    current_authority: StoreCurrentAuthorityIdentity,
) -> Result<(), BackupCutAdmissionDenial> {
    let mut roots = manifest
        .artifacts()
        .iter()
        .filter(|artifact| artifact.family() == super::BackupArtifactFamily::RootManifest);
    let root = roots
        .next()
        .ok_or(BackupCutAdmissionDenial::RootManifestCardinality)?;
    if roots.next().is_some() {
        return Err(BackupCutAdmissionDenial::RootManifestCardinality);
    }
    if root.generation() != coordinates.root_generation() {
        return Err(BackupCutAdmissionDenial::RootGenerationMismatch);
    }
    if !matches!(
        root.coverage(),
        BackupArtifactCoverage::RootManifest { root_generation }
            if *root_generation == coordinates.root_generation()
    ) {
        return Err(BackupCutAdmissionDenial::RootCoverageMismatch);
    }
    let mut checkpoints = manifest
        .artifacts()
        .iter()
        .filter(|artifact| artifact.family() == super::BackupArtifactFamily::CheckpointManifest);
    let checkpoint = checkpoints
        .next()
        .ok_or(BackupCutAdmissionDenial::CheckpointManifestCardinality)?;
    if checkpoints.next().is_some() {
        return Err(BackupCutAdmissionDenial::CheckpointManifestCardinality);
    }
    if checkpoint.identity() != coordinates.checkpoint_identity() {
        return Err(BackupCutAdmissionDenial::CheckpointIdentityMismatch);
    }
    if checkpoint.generation() != coordinates.manifest_generation() {
        return Err(BackupCutAdmissionDenial::CheckpointGenerationMismatch);
    }
    let BackupArtifactCoverage::CheckpointManifest {
        checkpoint_identity,
        manifest_generation,
        durable_checkpoint_lsn,
        authority_fingerprint,
        frontier_digest,
    } = checkpoint.coverage()
    else {
        return Err(BackupCutAdmissionDenial::CheckpointCoverageMismatch);
    };
    if checkpoint_identity != coordinates.checkpoint_identity()
        || *manifest_generation != coordinates.manifest_generation()
        || *durable_checkpoint_lsn != coordinates.durable_checkpoint_lsn()
    {
        return Err(BackupCutAdmissionDenial::CheckpointCoverageMismatch);
    }
    if *frontier_digest == [0; 32] {
        return Err(BackupCutAdmissionDenial::CheckpointFrontierMismatch);
    }
    if *authority_fingerprint != current_authority.fingerprint() {
        return Err(BackupCutAdmissionDenial::CheckpointFrontierMismatch);
    }
    validate_wal_coverage(coordinates, manifest)?;
    Ok(())
}

fn validate_wal_coverage(
    coordinates: &BackupCutCoordinates,
    manifest: &BackupCutManifest,
) -> Result<(), BackupCutAdmissionDenial> {
    let mut intervals = Vec::new();
    intervals
        .try_reserve_exact(manifest.artifacts().len())
        .map_err(|_| BackupCutAdmissionDenial::AllocationFailed)?;
    for artifact in manifest.artifacts() {
        if let BackupArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } = artifact.coverage()
        {
            intervals.push((*start_lsn, *end_exclusive_lsn));
        }
    }
    intervals.sort_unstable();
    let expected = coordinates.wal_half_open_interval();
    let Some(first) = intervals.first() else {
        return Err(BackupCutAdmissionDenial::WalCoverageGap);
    };
    if first.0 != expected.0 || intervals.last().map(|range| range.1) != Some(expected.1) {
        return Err(BackupCutAdmissionDenial::WalCoverageGap);
    }
    if intervals.windows(2).any(|pair| pair[0].1 != pair[1].0) {
        return Err(BackupCutAdmissionDenial::WalCoverageGap);
    }
    Ok(())
}

fn cut_identity(
    authority: StoreCurrentAuthorityIdentity,
    security_scope: StoreSecurityScopeAdmissionReceipt,
    coordinates: &BackupCutCoordinates,
    manifest: &BackupCutManifest,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store:admitted-backup-cut:v2\0");
    digest.update(authority.fingerprint());
    update_text(&mut digest, coordinates.store_lineage());
    digest.update(coordinates.root_generation().to_le_bytes());
    digest.update(coordinates.manifest_generation().to_le_bytes());
    update_text(&mut digest, coordinates.checkpoint_identity());
    digest.update(coordinates.durable_checkpoint_lsn().to_le_bytes());
    let (wal_start, wal_end) = coordinates.wal_half_open_interval();
    digest.update(wal_start.to_le_bytes());
    digest.update(wal_end.to_le_bytes());
    digest.update(coordinates.acknowledged_frontier().to_le_bytes());
    update_text(&mut digest, coordinates.format_profile());
    update_text(&mut digest, coordinates.backend_profile());
    digest.update(security_scope.identity().stable_fingerprint());
    let receipt = security_scope.receipt_id();
    digest.update(receipt.admission_sequence().to_le_bytes());
    digest.update(receipt.security_scope_fingerprint().to_le_bytes());
    digest.update(receipt.proof_progression_fingerprint().to_le_bytes());
    digest.update(manifest.artifact_closure_digest());
    for artifact in manifest.artifacts() {
        digest.update(artifact.physical_identity());
    }
    digest.finalize().into()
}

fn update_text(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value.as_bytes());
}
