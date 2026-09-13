use std::path::PathBuf;

use crate::{
    OperationalOperationId, OperationalSecurityScope, ProductionRestoreAdmissibleBackupBundle,
};
use sha2::{Digest, Sha256};
use worth_store_physical_isolation::{
    AdmittedPitrSourceCut, PitrReachabilityLease, RecoverySourceLeaseDenial,
    RecoverySourceLeaseRegistry, RecoverySourceLeaseRequest,
};

use super::{
    ExactRecoveryFrontier, PitrCandidateSelectionDenial, PointInTimeCandidate,
    PointInTimeCandidateSet,
};

#[derive(Debug)]
pub struct PointInTimeRecoveryIntent {
    operation_id: OperationalOperationId,
    backup: ProductionRestoreAdmissibleBackupBundle,
    candidates: PointInTimeCandidateSet,
    target_parent: PathBuf,
    security_scope: OperationalSecurityScope,
    admitted_capacity_bytes: u64,
    copy_buffer_bytes: usize,
}

impl PointInTimeRecoveryIntent {
    #[allow(clippy::too_many_arguments)]
    pub fn near(
        operation_id: OperationalOperationId,
        backup: ProductionRestoreAdmissibleBackupBundle,
        candidates: PointInTimeCandidateSet,
        target_parent: impl Into<PathBuf>,
        security_scope: OperationalSecurityScope,
        admitted_capacity_bytes: u64,
        copy_buffer_bytes: usize,
    ) -> Self {
        Self {
            operation_id,
            backup,
            candidates,
            target_parent: target_parent.into(),
            security_scope,
            admitted_capacity_bytes,
            copy_buffer_bytes,
        }
    }

    pub fn resolve(self) -> Result<ResolvedPitrCandidate, PitrResolutionDenial> {
        let candidate = self
            .candidates
            .select()
            .map_err(PitrResolutionDenial::Candidate)?;
        let materialized = self.backup.custody().structural().materialized();
        if candidate.source_identity() != materialized.manifest_digest() {
            return Err(PitrResolutionDenial::SourceIdentityMismatch);
        }
        if candidate.exact_frontier().authority_identity()
            != self.backup.admission().admitting_authority()
        {
            return Err(PitrResolutionDenial::SourceAuthorityMismatch);
        }
        let frontier = candidate.exact_frontier();
        let manifest = materialized.manifest();
        if frontier.checkpoint_durability() != manifest.durable_checkpoint_lsn()
            || frontier.wal_structural() < manifest.durable_checkpoint_lsn()
            || frontier.wal_structural() > manifest.wal_half_open_interval().1
            || frontier.local_durable_commit() > manifest.acknowledged_frontier()
            || frontier.client_acknowledged() < frontier.wal_structural()
            || frontier.client_acknowledged() > manifest.acknowledged_frontier()
            || frontier.replication_acknowledged() > manifest.acknowledged_frontier()
        {
            return Err(PitrResolutionDenial::FrontierOutsideSource);
        }
        Ok(ResolvedPitrCandidate {
            operation_id: self.operation_id,
            backup: self.backup,
            candidate,
            target_parent: self.target_parent,
            security_scope: self.security_scope,
            admitted_capacity_bytes: self.admitted_capacity_bytes,
            copy_buffer_bytes: self.copy_buffer_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitrResolutionDenial {
    Candidate(PitrCandidateSelectionDenial),
    SourceIdentityMismatch,
    SourceAuthorityMismatch,
    FrontierOutsideSource,
}

#[derive(Debug)]
pub struct ResolvedPitrCandidate {
    pub(super) operation_id: OperationalOperationId,
    pub(super) backup: ProductionRestoreAdmissibleBackupBundle,
    pub(super) candidate: PointInTimeCandidate,
    pub(super) target_parent: PathBuf,
    pub(super) security_scope: OperationalSecurityScope,
    pub(super) admitted_capacity_bytes: u64,
    pub(super) copy_buffer_bytes: usize,
}

impl ResolvedPitrCandidate {
    pub const fn exact_frontier(&self) -> ExactRecoveryFrontier {
        self.candidate.exact_frontier()
    }

    pub fn admit_source_cut(
        self,
        registry: &RecoverySourceLeaseRegistry,
    ) -> Result<AdmittedPitrSourceOperation, PitrSourceAdmissionDenial> {
        let materialized = self.backup.custody().structural().materialized();
        let mut artifacts = Vec::new();
        artifacts
            .try_reserve_exact(materialized.manifest().artifacts().len() + 1)
            .map_err(|_| PitrSourceAdmissionDenial::AllocationFailed)?;
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
            .admit_pitr_source_cut(request)
            .map_err(PitrSourceAdmissionDenial::Isolation)?;
        Ok(AdmittedPitrSourceOperation {
            resolved: self,
            admitted,
        })
    }
}

#[derive(Debug)]
pub enum PitrSourceAdmissionDenial {
    Isolation(RecoverySourceLeaseDenial),
    AllocationFailed,
}

#[derive(Debug)]
pub struct AdmittedPitrSourceOperation {
    resolved: ResolvedPitrCandidate,
    admitted: AdmittedPitrSourceCut,
}

impl AdmittedPitrSourceOperation {
    pub fn lease(self) -> EvidenceBoundPointInTimeRecoveryPlan {
        EvidenceBoundPointInTimeRecoveryPlan {
            resolved: self.resolved,
            lease: self.admitted.lease(),
        }
    }
}

#[derive(Debug)]
pub struct EvidenceBoundPointInTimeRecoveryPlan {
    pub(super) resolved: ResolvedPitrCandidate,
    pub(super) lease: PitrReachabilityLease,
}

pub(super) fn operation_identity(operation: &OperationalOperationId) -> [u8; 32] {
    Sha256::digest(operation.as_str().as_bytes()).into()
}
