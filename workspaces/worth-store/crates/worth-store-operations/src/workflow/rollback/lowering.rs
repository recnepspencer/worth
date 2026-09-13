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
use worth_store_physical_isolation::RollbackReachabilityLease;

use super::intent::operation_identity;
use super::{
    EvidenceBoundRollbackPlan, RollbackOperation, RollbackReplayDenial, RollbackReplayOwner,
    RollbackReplayPlan,
};

#[derive(Debug)]
pub enum RollbackLoweringDenial {
    LeaseBindingMismatch,
    InvalidArtifact,
    Backend(NonCurrentStagingLoweringDenial),
    Recovery(RollbackReplayDenial),
    OwnerDag(crate::OwnerPlanDagDenial),
    InvalidFootprint,
    InvalidOwnerVerification,
}

#[derive(Debug)]
pub struct LoweredRollbackPlanDag {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: LoweredOperationalPlan<RollbackOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: RollbackReplayPlan,
    pub(super) lease: RollbackReachabilityLease,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
    explanation: crate::CanonicalOwnerPlanDagExplanation,
}

#[derive(Debug)]
pub struct AuthorizedRollbackPlan {
    pub(super) operation_id: crate::OperationalOperationId,
    pub(super) authorization: AuthorizedOperationalPlan<RollbackOperation>,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: RollbackReplayPlan,
    pub(super) lease: RollbackReachabilityLease,
    pub(super) owner_verification: worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet,
}

impl EvidenceBoundRollbackPlan {
    pub fn lower(self) -> Result<LoweredRollbackPlanDag, RollbackLoweringDenial> {
        let resolved = self.resolved;
        let materialized = resolved.source.custody().structural().materialized();
        if self.lease.source_identity() != resolved.candidate.source_identity()
            || self.lease.source_root() != materialized.root()
        {
            return Err(RollbackLoweringDenial::LeaseBindingMismatch);
        }
        let mut artifacts = Vec::new();
        let manifest_path = materialized.root().join("backup.manifest");
        let manifest_bytes = std::fs::metadata(manifest_path)
            .map_err(|_| RollbackLoweringDenial::InvalidArtifact)?
            .len();
        artifacts.push(
            NonCurrentStagingArtifact::admit(
                "backup.manifest",
                manifest_bytes,
                materialized.manifest_digest(),
            )
            .ok_or(RollbackLoweringDenial::InvalidArtifact)?,
        );
        for row in materialized.manifest().artifacts() {
            artifacts.push(
                NonCurrentStagingArtifact::admit(
                    row.output_name(),
                    row.bytes(),
                    row.content_digest(),
                )
                .ok_or(RollbackLoweringDenial::InvalidArtifact)?,
            );
        }
        let backend = PhysicalRecoveryStagingOwner::lower(NonCurrentStagingPlanRequest::new(
            operation_identity(&resolved.operation_id),
            self.lease.source_root(),
            &resolved.target_parent,
            artifacts,
            resolved.admitted_capacity_bytes,
            resolved.copy_buffer_bytes,
        ))
        .map_err(RollbackLoweringDenial::Backend)?;
        let recovery = RollbackReplayOwner::lower(&resolved.candidate, backend.binding())
            .map_err(RollbackLoweringDenial::Recovery)?;
        let footprint = OwnerPlanFootprint::bounded(0, backend.binding().expected_bytes())
            .ok_or(RollbackLoweringDenial::InvalidFootprint)?;
        let owner_verification =
            worth_store_offline_verifier::StagedRecoveryOwnerVerificationSet::for_manifest(
                materialized.manifest(),
                materialized.manifest_digest(),
            )
            .ok_or(RollbackLoweringDenial::InvalidOwnerVerification)?;
        let owners = crate::workflow::recovery_owner_plan::lower_recovery_lifecycle_owners(
            backend.binding().fingerprint(),
            recovery.fingerprint(),
            footprint,
            owner_verification,
        )
        .map_err(RollbackLoweringDenial::OwnerDag)?;
        let binding = OperationalPlanBinding::bind(
            DestructiveOperationKind::Rollback,
            owners.dag,
            resolved.source.admission().admitting_authority(),
            resolved.security_scope,
            self.lease.binding_fingerprint(),
            path_identity(&resolved.target_parent),
            resolved.candidate.frontier().identity(),
        );
        Ok(LoweredRollbackPlanDag {
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

impl LoweredRollbackPlanDag {
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
    ) -> Result<AuthorizedRollbackPlan, AuthorizationDenial> {
        Ok(AuthorizedRollbackPlan {
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

fn path_identity(path: &std::path::Path) -> [u8; 32] {
    Sha256::digest(path.as_os_str().to_string_lossy().as_bytes()).into()
}
