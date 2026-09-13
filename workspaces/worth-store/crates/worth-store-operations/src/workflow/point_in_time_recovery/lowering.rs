use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    LoweredNonCurrentStagingPlan, NonCurrentStagingArtifact, NonCurrentStagingLoweringDenial,
    NonCurrentStagingPlanRequest, PhysicalRecoveryStagingOwner,
};
use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleArtifactManifestRow,
    BackupBundleFormatAuthority, BackupBundleFormatDenial, BackupBundleManifest,
    BackupBundleManifestConstructionDenial, BackupBundleManifestDeclaration,
    BackupBundleManifestIdentity, BackupBundleRecoveryCoordinates,
};
use worth_store_physical_isolation::PitrReachabilityLease;
use worth_store_wal::artifact_store::{
    inspect_wal_exact_frontier_prefix, WalExactFrontierPrefixDenial, WalExactFrontierPrefixRequest,
};

use crate::authorization::{
    authorize_lowered_plan, AuthorizationReplayPolicy, AuthorizedOperationalPlan,
    LoweredOperationalPlan,
};
use crate::owner_plan_dag::{DestructiveOperationKind, OperationalPlanBinding, OwnerPlanFootprint};
use crate::{
    AuthorizationDenial, AuthorizationRevocationObservation, ExternalOperatorAssertion,
    OperationalAuthorizationPort,
};

use super::intent::operation_identity;
use super::{
    EvidenceBoundPointInTimeRecoveryPlan, ExactRecoveryFrontier, PointInTimeRecoveryOperation,
    PointInTimeReplayDenial, PointInTimeReplayOwner, PointInTimeReplayPlan,
    PointInTimeReplayRequest, PointInTimeReplaySourceCoordinates,
};

#[derive(Debug)]
pub enum PitrLoweringDenial {
    LeaseBindingMismatch,
    MissingTargetWalFrame,
    WalPrefix(WalExactFrontierPrefixDenial),
    Manifest(BackupBundleManifestConstructionDenial),
    ManifestEncoding(BackupBundleFormatDenial),
    Backend(NonCurrentStagingLoweringDenial),
    Recovery(PointInTimeReplayDenial),
    OwnerDag(crate::OwnerPlanDagDenial),
    InvalidArtifact,
    InvalidFootprint,
    InvalidOwnerVerification,
}

#[derive(Debug)]
pub struct LoweredPointInTimeRecoveryPlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: LoweredOperationalPlan<PointInTimeRecoveryOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: PointInTimeReplayPlan,
    pub(super) lease: PitrReachabilityLease,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
    explanation: crate::CanonicalOwnerPlanDagExplanation,
}

#[derive(Debug)]
pub struct AuthorizedPointInTimeRecoveryPlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: AuthorizedOperationalPlan<PointInTimeRecoveryOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: PointInTimeReplayPlan,
    pub(super) lease: PitrReachabilityLease,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
}

impl EvidenceBoundPointInTimeRecoveryPlan {
    pub fn lower(self) -> Result<LoweredPointInTimeRecoveryPlan, PitrLoweringDenial> {
        let resolved = self.resolved;
        if self.lease.source_identity() != resolved.candidate.source_identity() {
            return Err(PitrLoweringDenial::LeaseBindingMismatch);
        }
        let materialized = resolved.backup.custody().structural().materialized();
        if self.lease.source_root() != materialized.root() {
            return Err(PitrLoweringDenial::LeaseBindingMismatch);
        }
        let (artifacts, exact_manifest, exact_manifest_digest) = exact_frontier_artifacts(
            materialized.manifest(),
            materialized.root(),
            resolved.candidate.exact_frontier(),
        )?;
        let backend = PhysicalRecoveryStagingOwner::lower(NonCurrentStagingPlanRequest::new(
            operation_identity(&resolved.operation_id),
            self.lease.source_root(),
            &resolved.target_parent,
            artifacts,
            resolved.admitted_capacity_bytes,
            resolved.copy_buffer_bytes,
        ))
        .map_err(PitrLoweringDenial::Backend)?;
        let frontier = resolved.candidate.exact_frontier();
        let recovery = PointInTimeReplayOwner::lower(PointInTimeReplayRequest::new(
            frontier,
            self.lease.source_identity(),
            backend.binding(),
            PointInTimeReplaySourceCoordinates {
                staged_manifest_digest: exact_manifest_digest,
                staged_wal_start: exact_manifest.wal_half_open_interval().0,
                source_checkpoint_lsn: exact_manifest.durable_checkpoint_lsn(),
                source_wal_end: materialized.manifest().wal_half_open_interval().1,
                source_acknowledged_frontier: materialized.manifest().acknowledged_frontier(),
            },
        ))
        .map_err(PitrLoweringDenial::Recovery)?;
        let footprint = OwnerPlanFootprint::bounded(0, backend.binding().expected_bytes())
            .ok_or(PitrLoweringDenial::InvalidFootprint)?;
        let owner_verification =
            worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet::for_manifest(
                &exact_manifest,
                exact_manifest_digest,
            )
            .ok_or(PitrLoweringDenial::InvalidOwnerVerification)?;
        let owners = crate::workflow::recovery_owner_plan::lower_recovery_lifecycle_owners(
            backend.binding().fingerprint(),
            recovery.fingerprint(),
            footprint,
            owner_verification,
        )
        .map_err(PitrLoweringDenial::OwnerDag)?;
        let binding = OperationalPlanBinding::bind(
            DestructiveOperationKind::PointInTimeRecovery,
            owners.dag,
            resolved.backup.admission().admitting_authority(),
            resolved.security_scope,
            self.lease.binding_fingerprint(),
            path_identity(&resolved.target_parent),
            resolved.candidate.identity(),
        );
        Ok(LoweredPointInTimeRecoveryPlan {
            operation_id: resolved.operation_id,
            authorization: LoweredOperationalPlan::from_binding(binding),
            backend,
            recovery,
            lease: self.lease,
            owner_verification: owners.verification,
            explanation: owners.explanation,
        })
    }
}

impl LoweredPointInTimeRecoveryPlan {
    pub const fn operation_id(&self) -> &crate::OperationalOperationId {
        &self.operation_id
    }
    pub const fn explanation(&self) -> &crate::CanonicalOwnerPlanDagExplanation {
        &self.explanation
    }

    #[allow(clippy::too_many_arguments)]
    pub fn authorize(
        self,
        port: &impl OperationalAuthorizationPort,
        assertion: &ExternalOperatorAssertion,
        requested_at: u64,
        expires_at: u64,
        replay_policy: AuthorizationReplayPolicy,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<AuthorizedPointInTimeRecoveryPlan, AuthorizationDenial> {
        Ok(AuthorizedPointInTimeRecoveryPlan {
            operation_id: self.operation_id,
            authorization: authorize_lowered_plan(
                self.authorization,
                port,
                assertion,
                requested_at,
                expires_at,
                replay_policy,
                revocation,
            )?,
            backend: self.backend,
            recovery: self.recovery,
            lease: self.lease,
            owner_verification: self.owner_verification,
        })
    }
}

fn exact_frontier_artifacts(
    manifest: &BackupBundleManifest,
    root: &std::path::Path,
    frontier: ExactRecoveryFrontier,
) -> Result<
    (
        Vec<NonCurrentStagingArtifact>,
        BackupBundleManifest,
        [u8; 32],
    ),
    PitrLoweringDenial,
> {
    let mut rows = Vec::new();
    let mut artifacts = Vec::new();
    let target = frontier.wal_structural();
    let mut found_target = target == manifest.durable_checkpoint_lsn();
    for row in manifest.artifacts() {
        match row.coverage() {
            BackupBundleArtifactCoverage::WalSegment {
                start_lsn,
                end_exclusive_lsn,
            } if *start_lsn < target && target < *end_exclusive_lsn => {
                let prefix = inspect_wal_exact_frontier_prefix(WalExactFrontierPrefixRequest::new(
                    root.join(row.output_name()),
                    row.reclaim_owner()
                        .generation_owner()
                        .and_then(|owner| owner.segment_id())
                        .map_or(0, |id| id.get()),
                    row.generation(),
                    target,
                    row.bytes(),
                ))
                .map_err(PitrLoweringDenial::WalPrefix)?;
                rows.push(prefix_row(row, prefix.bytes(), prefix.digest(), target)?);
                artifacts.push(
                    NonCurrentStagingArtifact::admit_prefix(
                        row.output_name(),
                        row.bytes(),
                        row.content_digest(),
                        prefix.bytes(),
                        prefix.digest(),
                    )
                    .ok_or(PitrLoweringDenial::InvalidArtifact)?,
                );
                found_target = true;
            }
            BackupBundleArtifactCoverage::WalSegment {
                end_exclusive_lsn, ..
            } if *end_exclusive_lsn <= target => {
                rows.push(row.clone());
                artifacts.push(full_artifact(row)?);
                found_target |= *end_exclusive_lsn == target;
            }
            BackupBundleArtifactCoverage::WalSegment { .. } => {}
            _ => {
                rows.push(row.clone());
                artifacts.push(full_artifact(row)?);
            }
        }
    }
    if !found_target {
        return Err(PitrLoweringDenial::MissingTargetWalFrame);
    }
    let exact = BackupBundleManifest::canonical_checked(BackupBundleManifestDeclaration::new(
        BackupBundleManifestIdentity {
            cut_identity: manifest.cut_identity(),
            store_lineage: format!("{}::pitr:{}", manifest.store_lineage(), target),
            root_generation: manifest.root_generation(),
            manifest_generation: manifest.manifest_generation(),
        },
        BackupBundleRecoveryCoordinates {
            checkpoint_identity: manifest.checkpoint_identity().to_owned(),
            durable_checkpoint_lsn: manifest.durable_checkpoint_lsn(),
            wal_half_open_interval: (manifest.wal_half_open_interval().0, target),
            acknowledged_frontier: frontier.client_acknowledged(),
        },
        manifest.security_scope_fingerprint(),
        rows,
    ))
    .map_err(PitrLoweringDenial::Manifest)?;
    let encoded = BackupBundleFormatAuthority::canonical()
        .encode_manifest(&exact)
        .map_err(PitrLoweringDenial::ManifestEncoding)?;
    let manifest_digest = Sha256::digest(&encoded).into();
    artifacts.push(
        NonCurrentStagingArtifact::admit_inline("backup.manifest", encoded)
            .ok_or(PitrLoweringDenial::InvalidArtifact)?,
    );
    Ok((artifacts, exact, manifest_digest))
}

fn prefix_row(
    row: &BackupBundleArtifactManifestRow,
    bytes: u64,
    digest: [u8; 32],
    target: u64,
) -> Result<BackupBundleArtifactManifestRow, PitrLoweringDenial> {
    let start = match row.coverage() {
        BackupBundleArtifactCoverage::WalSegment { start_lsn, .. } => *start_lsn,
        _ => return Err(PitrLoweringDenial::InvalidArtifact),
    };
    BackupBundleArtifactManifestRow::new(
        BackupBundleArtifactFamily::WalSegment,
        row.format(),
        format!("{}::prefix:{target}", row.identity()),
        row.output_name(),
        row.generation(),
        bytes,
        digest,
        BackupBundleArtifactCoverage::WalSegment {
            start_lsn: start,
            end_exclusive_lsn: target,
        },
        row.reclaim_owner(),
    )
    .ok_or(PitrLoweringDenial::InvalidArtifact)
}

fn full_artifact(
    row: &BackupBundleArtifactManifestRow,
) -> Result<NonCurrentStagingArtifact, PitrLoweringDenial> {
    NonCurrentStagingArtifact::admit(row.output_name(), row.bytes(), row.content_digest())
        .ok_or(PitrLoweringDenial::InvalidArtifact)
}

fn path_identity(path: &std::path::Path) -> [u8; 32] {
    Sha256::digest(path.as_os_str().to_string_lossy().as_bytes()).into()
}
