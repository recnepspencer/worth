use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactManifestRow,
};
use worth_store_security::StoreAuthorityBoundSecurityScopeReceipt;

use super::cut_recovery::{
    BackupCutReadmissionDenial, BackupCutRecoveryRecord, BackupCutRecoverySource,
};
use super::{
    AdmittedBackupCut, BackupArtifactCoverage, BackupArtifactReference,
    BackupCutAdmissionAuthority, BackupCutAdmissionRequest, BackupCutCoordinates,
    BackupCutManifest, BackupCutStoragePosture, UntrustedBackupArtifactClaim,
};
use crate::CurrentGenerationPhysicalReference;

impl BackupCutRecoveryRecord {
    pub fn readmit(
        &self,
        current_authority: &StoreCurrentAuthorityWitness,
        security_scope: StoreAuthorityBoundSecurityScopeReceipt,
        storage_posture: BackupCutStoragePosture,
        observation_buffer_bytes: usize,
    ) -> Result<AdmittedBackupCut, BackupCutReadmissionDenial> {
        self.validate_readmission_authority(current_authority, &security_scope)?;
        if self.sources.len() != self.manifest.artifacts().len() {
            return Err(BackupCutReadmissionDenial::MissingSource(copy_string(
                "canonical source closure",
            )?));
        }
        let mut artifacts = Vec::new();
        artifacts
            .try_reserve_exact(self.manifest.artifacts().len())
            .map_err(|_| BackupCutReadmissionDenial::AllocationFailed)?;
        for (row, source) in self.manifest.artifacts().iter().zip(&self.sources) {
            artifacts.push(recover_artifact(row, source, observation_buffer_bytes)?);
        }
        let manifest = BackupCutManifest::from_recovered_artifacts(
            artifacts,
            current_authority.authority_identity(),
        )
        .map_err(BackupCutReadmissionDenial::Manifest)?;
        if manifest.artifact_closure_digest() != self.manifest.artifact_closure_digest() {
            return Err(BackupCutReadmissionDenial::CutIdentityChanged);
        }
        let coordinates = BackupCutCoordinates::new(
            self.manifest.store_lineage(),
            self.manifest.root_generation(),
            self.manifest.manifest_generation(),
            self.manifest.checkpoint_identity(),
            self.manifest.durable_checkpoint_lsn(),
            self.manifest.wal_half_open_interval().0,
            self.manifest.wal_half_open_interval().1,
            self.manifest.acknowledged_frontier(),
            &self.format_profile,
            &self.backend_profile,
        )
        .ok_or(BackupCutReadmissionDenial::Coordinates)?;
        let admitted = BackupCutAdmissionAuthority::for_current_store(current_authority)
            .admit(BackupCutAdmissionRequest::new(
                security_scope,
                coordinates,
                manifest,
                storage_posture,
            ))
            .map_err(BackupCutReadmissionDenial::Admission)?;
        if admitted.identity() != self.cut_identity() {
            return Err(BackupCutReadmissionDenial::CutIdentityChanged);
        }
        Ok(admitted)
    }

    fn validate_readmission_authority(
        &self,
        current_authority: &StoreCurrentAuthorityWitness,
        security_scope: &StoreAuthorityBoundSecurityScopeReceipt,
    ) -> Result<(), BackupCutReadmissionDenial> {
        if current_authority.authority_identity() != self.authority_identity {
            return Err(BackupCutReadmissionDenial::AuthorityIdentityChanged);
        }
        if security_scope.authority_identity() != current_authority.authority_identity() {
            return Err(BackupCutReadmissionDenial::SecurityScopeAuthorityChanged);
        }
        let security = security_scope.receipt().receipt_id();
        if security.security_scope_fingerprint() != self.manifest.security_scope_fingerprint() {
            return Err(BackupCutReadmissionDenial::SecurityScopeChanged);
        }
        if security.proof_progression_fingerprint() != self.security_progression_fingerprint {
            return Err(BackupCutReadmissionDenial::SecurityProofProgressionChanged);
        }
        Ok(())
    }
}

fn recover_artifact(
    row: &BackupBundleArtifactManifestRow,
    source: &BackupCutRecoverySource,
    buffer_bytes: usize,
) -> Result<BackupArtifactReference, BackupCutReadmissionDenial> {
    if row.output_name() != source.output_name {
        return Err(BackupCutReadmissionDenial::MissingSource(copy_string(
            row.output_name(),
        )?));
    }
    let observation = match worth_store_physical_backend::observe_physical_backup_artifact(
        &source.path,
        buffer_bytes,
    ) {
        Ok(observation) => observation,
        Err(source) => {
            return Err(BackupCutReadmissionDenial::SourceObservation {
                output_name: copy_string(row.output_name())?,
                source,
            });
        }
    };
    validate_observation(row, source, &observation)?;
    let Some(owner) = row.reclaim_owner().generation_owner() else {
        return Err(BackupCutReadmissionDenial::ArtifactInvariant(copy_string(
            row.output_name(),
        )?));
    };
    let Some(current) = CurrentGenerationPhysicalReference::from_durable_owner(owner) else {
        return Err(BackupCutReadmissionDenial::ArtifactInvariant(copy_string(
            row.output_name(),
        )?));
    };
    let artifact = BackupArtifactReference::declare_untrusted_physical_observation(
        UntrustedBackupArtifactClaim {
            family: recover_family(row.family()),
            format: row.format(),
            identity: copy_string(row.identity())?,
            generation: row.generation(),
            coverage: recover_coverage(row.coverage())?,
        },
        observation,
        current,
    );
    match artifact {
        Some(artifact) => Ok(artifact),
        None => Err(BackupCutReadmissionDenial::ArtifactInvariant(copy_string(
            row.output_name(),
        )?)),
    }
}

fn validate_observation(
    row: &BackupBundleArtifactManifestRow,
    source: &BackupCutRecoverySource,
    observation: &worth_store_physical_backend::PhysicalBackupArtifactObservation,
) -> Result<(), BackupCutReadmissionDenial> {
    let name = || copy_string(row.output_name());
    if observation.bytes() != row.bytes() {
        return Err(BackupCutReadmissionDenial::SourceLengthChanged(name()?));
    }
    if observation.content_digest() != row.content_digest() {
        return Err(BackupCutReadmissionDenial::SourceDigestChanged(name()?));
    }
    if observation.physical_identity() != source.physical_identity {
        return Err(BackupCutReadmissionDenial::SourcePhysicalIdentityChanged(
            name()?,
        ));
    }
    Ok(())
}

const fn recover_family(family: BackupBundleArtifactFamily) -> super::BackupArtifactFamily {
    use super::BackupArtifactFamily as Artifact;
    match family {
        BackupBundleArtifactFamily::RootManifest => Artifact::RootManifest,
        BackupBundleArtifactFamily::CheckpointManifest => Artifact::CheckpointManifest,
        BackupBundleArtifactFamily::WalSegment => Artifact::WalSegment,
        BackupBundleArtifactFamily::Page => Artifact::Page,
        BackupBundleArtifactFamily::Extent => Artifact::Extent,
        BackupBundleArtifactFamily::Index => Artifact::Index,
        BackupBundleArtifactFamily::BlobChunk => Artifact::BlobChunk,
        BackupBundleArtifactFamily::SecondaryRoot => Artifact::SecondaryRoot,
    }
}

fn recover_coverage(
    coverage: &BackupBundleArtifactCoverage,
) -> Result<BackupArtifactCoverage, BackupCutReadmissionDenial> {
    Ok(match coverage {
        BackupBundleArtifactCoverage::RootManifest { root_generation } => {
            BackupArtifactCoverage::RootManifest {
                root_generation: *root_generation,
            }
        }
        BackupBundleArtifactCoverage::CheckpointManifest {
            checkpoint_identity,
            manifest_generation,
            durable_checkpoint_lsn,
            authority_fingerprint,
            frontier_digest,
        } => BackupArtifactCoverage::CheckpointManifest {
            checkpoint_identity: copy_string(checkpoint_identity)?,
            manifest_generation: *manifest_generation,
            durable_checkpoint_lsn: *durable_checkpoint_lsn,
            authority_fingerprint: *authority_fingerprint,
            frontier_digest: *frontier_digest,
        },
        BackupBundleArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } => BackupArtifactCoverage::WalSegment {
            start_lsn: *start_lsn,
            end_exclusive_lsn: *end_exclusive_lsn,
        },
        BackupBundleArtifactCoverage::PhysicalReachability => {
            BackupArtifactCoverage::PhysicalReachability
        }
        BackupBundleArtifactCoverage::SecondaryRoot { root_generation } => {
            BackupArtifactCoverage::SecondaryRoot {
                root_generation: *root_generation,
            }
        }
    })
}

fn copy_string(value: &str) -> Result<String, BackupCutReadmissionDenial> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| BackupCutReadmissionDenial::AllocationFailed)?;
    copied.push_str(value);
    Ok(copied)
}
