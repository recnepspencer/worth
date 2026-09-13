use std::path::PathBuf;

use worth_store_physical_backend::{PhysicalBackupMaterializationSession, PhysicalBackupSource};
use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactManifestRow,
    BackupBundleFormatAuthority, BackupBundleManifest, BackupBundleManifestConstructionDenial,
    BackupBundleManifestDeclaration, BackupBundleManifestIdentity, BackupBundlePhysicalOwner,
    BackupBundleRecoveryCoordinates,
};
use worth_store_physical_isolation::{
    AdmittedBackupCut, BackupArtifactCoverage, BackupArtifactFamily, BackupArtifactReference,
    BackupReachabilityLeaseRegistry, BackupReachabilityLeaseRegistryDenial,
};

use crate::{
    BackupMaterializationRecoveryPlan, OperationalControlAppendDenial, OperationalControlRecord,
    OperationalControlStorePort, OperationalOperationId,
};

use super::{hex, transition, BackupMaterializationDenial, BackupMaterializationSession};

#[derive(Debug, PartialEq, Eq)]
pub struct AdmittedOnlineBackup {
    operation_id: OperationalOperationId,
    cut: AdmittedBackupCut,
    source_verification: worth_store_offline_verifier::BackupCutSourceVerificationReport,
}

impl AdmittedOnlineBackup {
    pub(super) const fn new(
        operation_id: OperationalOperationId,
        cut: AdmittedBackupCut,
        source_verification: worth_store_offline_verifier::BackupCutSourceVerificationReport,
    ) -> Self {
        Self {
            operation_id,
            cut,
            source_verification,
        }
    }
    pub const fn cut(&self) -> &AdmittedBackupCut {
        &self.cut
    }
    pub const fn source_verification(
        &self,
    ) -> &worth_store_offline_verifier::BackupCutSourceVerificationReport {
        &self.source_verification
    }

    pub fn abandon(
        self,
        reason: impl Into<String>,
        control: &impl OperationalControlStorePort,
        leases: &BackupReachabilityLeaseRegistry,
    ) -> Result<worth_store_physical_isolation::BackupCutAbandonmentReceipt, BackupAbandonmentDenial>
    {
        let reason = reason.into();
        let released = match record_durable_abandonment(
            &self.operation_id,
            &self.cut,
            &reason,
            control,
            leases,
        ) {
            Ok(released) => released,
            Err(source) => {
                return Err(BackupAbandonmentDenial {
                    backup: self,
                    source,
                })
            }
        };
        let Self {
            operation_id,
            cut,
            source_verification,
        } = self;
        let prepared =
            match worth_store_physical_isolation::prepare_backup_cut_abandonment(cut, released) {
                Ok(prepared) => prepared,
                Err(mismatch) => {
                    let (cut, released) = mismatch.into_parts();
                    return Err(BackupAbandonmentDenial {
                        backup: Self {
                            operation_id,
                            cut,
                            source_verification,
                        },
                        source: BackupAbandonmentFailure::ReleasedCutMismatch(released),
                    });
                }
            };
        Ok(worth_store_physical_isolation::abandon_backup_cut(
            prepared, reason,
        ))
    }

    pub fn materialize<'a>(
        self,
        target_parent: impl Into<PathBuf>,
        buffer_bytes: usize,
        control: &'a impl OperationalControlStorePort,
    ) -> Result<BackupMaterializationSession<'a>, BackupMaterializationDenial> {
        let target_parent = target_parent.into();
        let (sources, rows) =
            prepare_materialization_declarations(self.cut.manifest().artifacts())?;
        let coordinates = self.cut.coordinates();
        let manifest =
            BackupBundleManifest::canonical_checked(BackupBundleManifestDeclaration::new(
                BackupBundleManifestIdentity {
                    cut_identity: self.cut.identity(),
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
                self.cut
                    .security_scope()
                    .receipt_id()
                    .security_scope_fingerprint(),
                rows,
            ))
            .map_err(|denial| match denial {
                BackupBundleManifestConstructionDenial::InvalidManifest => {
                    BackupMaterializationDenial::AdmittedCutInvariant
                }
                BackupBundleManifestConstructionDenial::AllocationFailed => {
                    BackupMaterializationDenial::PreparationAllocationFailed
                }
            })?;
        let plan = BackupMaterializationRecoveryPlan::prepare(
            self.cut.identity(),
            &target_parent,
            buffer_bytes,
        )
        .map_err(BackupMaterializationDenial::Plan)?;
        let plan_record = OperationalControlRecord::backup_materialization_opened(
            self.cut.authority_identity(),
            self.operation_id.clone(),
            transition(&self.operation_id, "materialization-opened"),
            plan.clone(),
        );
        control
            .append(&plan_record)
            .map_err(BackupMaterializationDenial::PlanPersistence)?;
        let physical = PhysicalBackupMaterializationSession::open_or_resume(
            plan.target_parent(),
            &hex(&self.cut.identity()),
            sources,
            plan.buffer_bytes(),
        )
        .map_err(BackupMaterializationDenial::Physical)?;
        Ok(BackupMaterializationSession::new(
            self.operation_id,
            self.cut,
            manifest,
            physical,
            control,
            BackupBundleFormatAuthority::canonical(),
        ))
    }
}

fn prepare_materialization_declarations(
    artifacts: &[BackupArtifactReference],
) -> Result<
    (
        Vec<PhysicalBackupSource>,
        Vec<BackupBundleArtifactManifestRow>,
    ),
    BackupMaterializationDenial,
> {
    let mut sources = Vec::new();
    let mut rows = Vec::new();
    sources
        .try_reserve_exact(artifacts.len())
        .map_err(|_| BackupMaterializationDenial::PreparationAllocationFailed)?;
    rows.try_reserve_exact(artifacts.len())
        .map_err(|_| BackupMaterializationDenial::PreparationAllocationFailed)?;
    for (index, artifact) in artifacts.iter().enumerate() {
        let output_name = output_name(index, artifact)?;
        sources.push(
            PhysicalBackupSource::new(
                artifact.source_path(),
                output_name.clone(),
                artifact.bytes(),
                artifact.content_digest(),
                artifact.physical_identity(),
            )
            .ok_or(BackupMaterializationDenial::AdmittedCutInvariant)?,
        );
        rows.push(
            BackupBundleArtifactManifestRow::new(
                family(artifact.family()),
                artifact.format(),
                artifact.identity(),
                output_name,
                artifact.generation(),
                artifact.bytes(),
                artifact.content_digest(),
                coverage(artifact.coverage()),
                BackupBundlePhysicalOwner::from_generation_owner(
                    artifact.reclaim_reference().owner(),
                ),
            )
            .ok_or(BackupMaterializationDenial::AdmittedCutInvariant)?,
        );
    }
    Ok((sources, rows))
}

#[derive(Debug)]
pub struct BackupAbandonmentDenial {
    backup: AdmittedOnlineBackup,
    source: BackupAbandonmentFailure,
}

#[derive(Debug)]
pub enum BackupAbandonmentFailure {
    Control(OperationalControlAppendDenial),
    Registry(BackupReachabilityLeaseRegistryDenial),
    ReleasedCutMismatch(worth_store_physical_isolation::ReleasedBackupReachabilityLease),
}

impl BackupAbandonmentDenial {
    pub fn into_retry(self) -> (AdmittedOnlineBackup, BackupAbandonmentFailure) {
        (self.backup, self.source)
    }
}

pub(super) fn record_durable_abandonment(
    operation_id: &OperationalOperationId,
    cut: &AdmittedBackupCut,
    reason: &str,
    control: &dyn OperationalControlStorePort,
    leases: &BackupReachabilityLeaseRegistry,
) -> Result<worth_store_physical_isolation::ReleasedBackupReachabilityLease, BackupAbandonmentFailure>
{
    let release_record = cut.lease().release_record();
    let holder = crate::control_store::backup_lease_holder_id(operation_id);
    let release_reservation = leases
        .reserve_release(holder, release_record.cut_identity())
        .map_err(BackupAbandonmentFailure::Registry)?;
    let record = OperationalControlRecord::backup_abandoned(
        cut.authority_identity(),
        operation_id.clone(),
        super::transition(operation_id, "abandoned"),
        reason,
        release_record,
    );
    let receipt = control
        .append(&record)
        .map_err(BackupAbandonmentFailure::Control)?;
    release_reservation
        .acknowledge_durable_release(receipt)
        .map_err(BackupAbandonmentFailure::Registry)
}

fn output_name(
    index: usize,
    artifact: &BackupArtifactReference,
) -> Result<String, BackupMaterializationDenial> {
    use std::fmt::Write;

    let mut output = String::new();
    output
        .try_reserve_exact(80)
        .map_err(|_| BackupMaterializationDenial::PreparationAllocationFailed)?;
    write!(output, "{index:04}-{}-", family_label(artifact.family()))
        .map_err(|_| BackupMaterializationDenial::PreparationAllocationFailed)?;
    for byte in &artifact.content_digest()[..8] {
        write!(output, "{byte:02x}")
            .map_err(|_| BackupMaterializationDenial::PreparationAllocationFailed)?;
    }
    output.push_str(".bin");
    Ok(output)
}

fn coverage(coverage: &BackupArtifactCoverage) -> BackupBundleArtifactCoverage {
    match coverage {
        BackupArtifactCoverage::RootManifest { root_generation } => {
            BackupBundleArtifactCoverage::RootManifest {
                root_generation: *root_generation,
            }
        }
        BackupArtifactCoverage::CheckpointManifest {
            checkpoint_identity,
            manifest_generation,
            durable_checkpoint_lsn,
            authority_fingerprint,
            frontier_digest,
        } => BackupBundleArtifactCoverage::CheckpointManifest {
            checkpoint_identity: checkpoint_identity.clone(),
            manifest_generation: *manifest_generation,
            durable_checkpoint_lsn: *durable_checkpoint_lsn,
            authority_fingerprint: *authority_fingerprint,
            frontier_digest: *frontier_digest,
        },
        BackupArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } => BackupBundleArtifactCoverage::WalSegment {
            start_lsn: *start_lsn,
            end_exclusive_lsn: *end_exclusive_lsn,
        },
        BackupArtifactCoverage::PhysicalReachability => {
            BackupBundleArtifactCoverage::PhysicalReachability
        }
        BackupArtifactCoverage::SecondaryRoot { root_generation } => {
            BackupBundleArtifactCoverage::SecondaryRoot {
                root_generation: *root_generation,
            }
        }
    }
}

const fn family(family: BackupArtifactFamily) -> BackupBundleArtifactFamily {
    match family {
        BackupArtifactFamily::RootManifest => BackupBundleArtifactFamily::RootManifest,
        BackupArtifactFamily::CheckpointManifest => BackupBundleArtifactFamily::CheckpointManifest,
        BackupArtifactFamily::WalSegment => BackupBundleArtifactFamily::WalSegment,
        BackupArtifactFamily::Page => BackupBundleArtifactFamily::Page,
        BackupArtifactFamily::Extent => BackupBundleArtifactFamily::Extent,
        BackupArtifactFamily::Index => BackupBundleArtifactFamily::Index,
        BackupArtifactFamily::BlobChunk => BackupBundleArtifactFamily::BlobChunk,
        BackupArtifactFamily::SecondaryRoot => BackupBundleArtifactFamily::SecondaryRoot,
    }
}

const fn family_label(family: BackupArtifactFamily) -> &'static str {
    match family {
        BackupArtifactFamily::RootManifest => "root-manifest",
        BackupArtifactFamily::CheckpointManifest => "checkpoint-manifest",
        BackupArtifactFamily::WalSegment => "wal",
        BackupArtifactFamily::Page => "page",
        BackupArtifactFamily::Extent => "extent",
        BackupArtifactFamily::Index => "index",
        BackupArtifactFamily::BlobChunk => "blob",
        BackupArtifactFamily::SecondaryRoot => "secondary-root",
    }
}
