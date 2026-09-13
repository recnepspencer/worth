use std::path::PathBuf;

use crate::workflow::point_in_time_recovery::ExactRecoveryFrontier;
use crate::{
    OperationalOperationId, OperationalSecurityScope, ProductionRestoreAdmissibleBackupBundle,
};
use sha2::{Digest, Sha256};
use worth_store_authority::StoreRetainedAuthorityEvidence;
use worth_store_physical_isolation::{
    AdmittedRollbackSourceCut, RecoverySourceLeaseDenial, RecoverySourceLeaseRegistry,
    RecoverySourceLeaseRequest, RollbackReachabilityLease,
};

use super::{ResolvedRollbackCandidate, RollbackReplayDenial, RollbackReplayOwner};

#[derive(Debug)]
pub struct RollbackIntent {
    operation_id: OperationalOperationId,
    retained: StoreRetainedAuthorityEvidence,
    source: ProductionRestoreAdmissibleBackupBundle,
    frontier: ExactRecoveryFrontier,
    target_parent: PathBuf,
    security_scope: OperationalSecurityScope,
    admitted_capacity_bytes: u64,
    copy_buffer_bytes: usize,
}

impl RollbackIntent {
    #[allow(clippy::too_many_arguments)]
    pub fn from_retained_authority(
        operation_id: OperationalOperationId,
        retained: StoreRetainedAuthorityEvidence,
        source: ProductionRestoreAdmissibleBackupBundle,
        frontier: ExactRecoveryFrontier,
        target_parent: impl Into<PathBuf>,
        security_scope: OperationalSecurityScope,
        admitted_capacity_bytes: u64,
        copy_buffer_bytes: usize,
    ) -> Self {
        Self {
            operation_id,
            retained,
            source,
            frontier,
            target_parent: target_parent.into(),
            security_scope,
            admitted_capacity_bytes,
            copy_buffer_bytes,
        }
    }

    pub fn resolve(self) -> Result<ResolvedRollbackOperation, RollbackResolutionDenial> {
        let materialized = self.source.custody().structural().materialized();
        let candidate = RollbackReplayOwner::resolve_candidate(
            &self.retained,
            materialized.manifest(),
            materialized.manifest_digest(),
            self.frontier,
        )
        .map_err(RollbackResolutionDenial::RecoveryPhysics)?;
        Ok(ResolvedRollbackOperation {
            operation_id: self.operation_id,
            source: self.source,
            candidate,
            target_parent: self.target_parent,
            security_scope: self.security_scope,
            admitted_capacity_bytes: self.admitted_capacity_bytes,
            copy_buffer_bytes: self.copy_buffer_bytes,
        })
    }
}

#[derive(Debug)]
pub enum RollbackResolutionDenial {
    RecoveryPhysics(RollbackReplayDenial),
}

#[derive(Debug)]
pub struct ResolvedRollbackOperation {
    pub(super) operation_id: OperationalOperationId,
    pub(super) source: ProductionRestoreAdmissibleBackupBundle,
    pub(super) candidate: ResolvedRollbackCandidate,
    pub(super) target_parent: PathBuf,
    pub(super) security_scope: OperationalSecurityScope,
    pub(super) admitted_capacity_bytes: u64,
    pub(super) copy_buffer_bytes: usize,
}

impl ResolvedRollbackOperation {
    pub const fn candidate(&self) -> &ResolvedRollbackCandidate {
        &self.candidate
    }

    pub fn admit_source_cut(
        self,
        registry: &RecoverySourceLeaseRegistry,
    ) -> Result<AdmittedRollbackSourceOperation, RollbackSourceAdmissionDenial> {
        let materialized = self.source.custody().structural().materialized();
        let mut artifacts = Vec::new();
        artifacts
            .try_reserve_exact(materialized.manifest().artifacts().len() + 1)
            .map_err(|_| RollbackSourceAdmissionDenial::AllocationFailed)?;
        artifacts.push("backup.manifest".to_owned());
        artifacts.extend(
            materialized
                .manifest()
                .artifacts()
                .iter()
                .map(|row| row.output_name().to_owned()),
        );
        let request = RecoverySourceLeaseRequest::new(
            operation_identity(&self.operation_id),
            self.candidate.source_identity(),
            materialized.root(),
            artifacts,
        );
        let admitted = registry
            .admit_rollback_source_cut(request)
            .map_err(RollbackSourceAdmissionDenial::Isolation)?;
        Ok(AdmittedRollbackSourceOperation {
            resolved: self,
            admitted,
        })
    }
}

#[derive(Debug)]
pub enum RollbackSourceAdmissionDenial {
    Isolation(RecoverySourceLeaseDenial),
    AllocationFailed,
}

#[derive(Debug)]
pub struct AdmittedRollbackSourceOperation {
    resolved: ResolvedRollbackOperation,
    admitted: AdmittedRollbackSourceCut,
}

impl AdmittedRollbackSourceOperation {
    pub fn lease(self) -> EvidenceBoundRollbackPlan {
        EvidenceBoundRollbackPlan {
            resolved: self.resolved,
            lease: self.admitted.lease(),
        }
    }
}

#[derive(Debug)]
pub struct EvidenceBoundRollbackPlan {
    pub(super) resolved: ResolvedRollbackOperation,
    pub(super) lease: RollbackReachabilityLease,
}

pub(super) fn operation_identity(operation: &OperationalOperationId) -> [u8; 32] {
    Sha256::digest(operation.as_str().as_bytes()).into()
}
