use crate::authorization::{
    authorize_lowered_plan, AuthorizationReplayPolicy, AuthorizedOperationalPlan,
    LoweredOperationalPlan,
};
use crate::owner_plan_dag::{DestructiveOperationKind, OperationalPlanBinding, OwnerPlanFootprint};
use crate::{
    AuthorizationDenial, AuthorizationRevocationObservation, ExternalOperatorAssertion,
    OperationalAuthorizationPort,
};
use sha2::{Digest, Sha256};
use worth_store_physical_backend::{
    LoweredNonCurrentStagingPlan, NonCurrentStagingArtifact, NonCurrentStagingLoweringDenial,
    NonCurrentStagingPlanRequest, PhysicalRecoveryStagingOwner,
};

use super::{
    BackupRestoreOperation, BackupRestoreReplayDenial, BackupRestoreReplayOwner,
    BackupRestoreReplayPlan, BackupRestoreReplayRequest, EvidenceBoundBackupRestorePlan,
};

#[derive(Debug)]
pub enum BackupRestoreLoweringDenial {
    SourceArtifactUnavailable { output_name: String },
    InvalidSourceArtifact { output_name: String },
    Backend(NonCurrentStagingLoweringDenial),
    Recovery(BackupRestoreReplayDenial),
    OwnerDag(crate::OwnerPlanDagDenial),
    InvalidFootprint,
    CounterOverflow,
    InvalidOwnerVerification,
}

#[derive(Debug, Clone)]
pub struct LoweredBackupRestorePlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: LoweredOperationalPlan<BackupRestoreOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: BackupRestoreReplayPlan,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
    explanation: crate::CanonicalOwnerPlanDagExplanation,
}

#[derive(Debug)]
pub struct AuthorizedBackupRestorePlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: AuthorizedOperationalPlan<BackupRestoreOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: BackupRestoreReplayPlan,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
}

impl EvidenceBoundBackupRestorePlan {
    pub fn lower(self) -> Result<LoweredBackupRestorePlan, BackupRestoreLoweringDenial> {
        let artifacts = staging_artifacts(&self)?;
        let operation_identity = operation_identity(&self.operation_id);
        let backend = PhysicalRecoveryStagingOwner::lower(NonCurrentStagingPlanRequest::new(
            operation_identity,
            self.source_root(),
            self.target_parent.clone(),
            artifacts,
            self.admitted_capacity_bytes,
            self.copy_buffer_bytes,
        ))
        .map_err(BackupRestoreLoweringDenial::Backend)?;
        let manifest = self.backup.custody().structural().materialized().manifest();
        let recovery =
            BackupRestoreReplayOwner::lower(BackupRestoreReplayRequest::from_verified_backup(
                manifest,
                self.source_identity,
                backend.binding(),
            ))
            .map_err(BackupRestoreLoweringDenial::Recovery)?;
        let footprint = OwnerPlanFootprint::bounded(0, backend.binding().expected_bytes())
            .ok_or(BackupRestoreLoweringDenial::InvalidFootprint)?;
        let owner_verification =
            worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet::for_manifest(
                manifest,
                self.backup
                    .custody()
                    .structural()
                    .materialized()
                    .manifest_digest(),
            )
            .ok_or(BackupRestoreLoweringDenial::InvalidOwnerVerification)?;
        let owners = crate::workflow::recovery_owner_plan::lower_recovery_lifecycle_owners(
            backend.binding().fingerprint(),
            recovery.fingerprint(),
            footprint,
            owner_verification,
        )
        .map_err(BackupRestoreLoweringDenial::OwnerDag)?;
        let frontier_identity =
            receipt_fingerprint(b"backup-restore-frontier", recovery.fingerprint());
        let binding = OperationalPlanBinding::bind(
            DestructiveOperationKind::BackupRestore,
            owners.dag,
            self.backup.admission().admitting_authority(),
            self.security_scope,
            self.source_identity,
            self.target_identity,
            frontier_identity,
        );
        Ok(LoweredBackupRestorePlan {
            operation_id: self.operation_id,
            authorization: LoweredOperationalPlan::from_binding(binding),
            backend,
            recovery,
            owner_verification: owners.verification,
            explanation: owners.explanation,
        })
    }
}

impl LoweredBackupRestorePlan {
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
    ) -> Result<AuthorizedBackupRestorePlan, AuthorizationDenial> {
        Ok(AuthorizedBackupRestorePlan {
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
            owner_verification: self.owner_verification,
        })
    }
}

fn staging_artifacts(
    plan: &EvidenceBoundBackupRestorePlan,
) -> Result<Vec<NonCurrentStagingArtifact>, BackupRestoreLoweringDenial> {
    let materialized = plan.backup.custody().structural().materialized();
    let mut artifacts = Vec::new();
    artifacts
        .try_reserve_exact(materialized.manifest().artifacts().len().saturating_add(1))
        .map_err(|_| BackupRestoreLoweringDenial::CounterOverflow)?;
    let manifest_path = materialized.root().join("backup.manifest");
    let manifest_bytes = std::fs::metadata(&manifest_path)
        .map_err(|_| BackupRestoreLoweringDenial::SourceArtifactUnavailable {
            output_name: "backup.manifest".into(),
        })?
        .len();
    artifacts.push(
        NonCurrentStagingArtifact::admit(
            "backup.manifest",
            manifest_bytes,
            materialized.manifest_digest(),
        )
        .ok_or_else(|| BackupRestoreLoweringDenial::InvalidSourceArtifact {
            output_name: "backup.manifest".into(),
        })?,
    );
    for row in materialized.manifest().artifacts() {
        artifacts.push(
            NonCurrentStagingArtifact::admit(row.output_name(), row.bytes(), row.content_digest())
                .ok_or_else(|| BackupRestoreLoweringDenial::InvalidSourceArtifact {
                    output_name: row.output_name().to_owned(),
                })?,
        );
    }
    Ok(artifacts)
}

fn operation_identity(operation: &crate::OperationalOperationId) -> [u8; 32] {
    Sha256::digest(operation.as_str().as_bytes()).into()
}

fn receipt_fingerprint(domain: &[u8], plan: [u8; 32]) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(plan);
    digest.finalize().into()
}
