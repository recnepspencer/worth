use sha2::{Digest, Sha256};
use worth_store_authority::{StoreCurrentAuthorityIdentity, StoreRetainedAuthorityEvidence};
use worth_store_physical_backend::{NonCurrentStagingMutationScope, NonCurrentStagingPlanBinding};
use worth_store_physical_format::BackupBundleManifest;

use crate::workflow::point_in_time_recovery::ExactRecoveryFrontier;
use crate::workflow::recovery_replay::{
    apply_staged_wal, validate_staged_wal_replay_source, StagedWalApplicationDenial,
    StagedWalApplicationPort, StagedWalApplicationReceipt, StagedWalReplaySourceDenial,
    StagedWalReplaySourceReceipt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRollbackCandidate {
    retained_authority: StoreCurrentAuthorityIdentity,
    source_identity: [u8; 32],
    source_lineage: [u8; 32],
    frontier: ExactRecoveryFrontier,
    root_generation: u64,
    manifest_digest: [u8; 32],
    wal_start_lsn: u64,
}

impl ResolvedRollbackCandidate {
    pub const fn retained_authority(&self) -> StoreCurrentAuthorityIdentity {
        self.retained_authority
    }
    pub const fn source_identity(&self) -> [u8; 32] {
        self.source_identity
    }
    pub const fn source_lineage(&self) -> [u8; 32] {
        self.source_lineage
    }
    pub const fn frontier(&self) -> ExactRecoveryFrontier {
        self.frontier
    }
    pub const fn root_generation(&self) -> u64 {
        self.root_generation
    }
    pub const fn manifest_digest(&self) -> [u8; 32] {
        self.manifest_digest
    }
}

#[derive(Debug)]
pub enum RollbackReplayDenial {
    RetainedAuthorityMismatch,
    InvalidSource,
    StagingPlanMismatch,
    EmptyStaging,
    ReplaySource(StagedWalReplaySourceDenial),
    Application(StagedWalApplicationDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollbackReplayPlan {
    fingerprint: [u8; 32],
    staging_plan_fingerprint: [u8; 32],
    frontier: ExactRecoveryFrontier,
    source_identity: [u8; 32],
    retained_authority: StoreCurrentAuthorityIdentity,
    manifest_digest: [u8; 32],
    wal_start_lsn: u64,
}

impl RollbackReplayPlan {
    pub const fn fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }
    pub const fn frontier(self) -> ExactRecoveryFrontier {
        self.frontier
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollbackExecutionReceipt {
    plan_fingerprint: [u8; 32],
    frontier: ExactRecoveryFrontier,
    replay_source: StagedWalReplaySourceReceipt,
    application: StagedWalApplicationReceipt,
}

impl RollbackExecutionReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn frontier(self) -> ExactRecoveryFrontier {
        self.frontier
    }
    pub const fn replay_source(self) -> StagedWalReplaySourceReceipt {
        self.replay_source
    }
    pub const fn application(self) -> StagedWalApplicationReceipt {
        self.application
    }
}

impl worth_store_physical_backend::NonCurrentStagingOwnerEffect for RollbackExecutionReceipt {
    fn effect_fingerprint(&self) -> [u8; 32] {
        self.application.identity()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RollbackReplayOwner;

impl RollbackReplayOwner {
    pub fn source_lineage(
        retained: &StoreRetainedAuthorityEvidence,
        manifest: &BackupBundleManifest,
    ) -> [u8; 32] {
        retained_source_lineage(
            StoreCurrentAuthorityIdentity::from_aspect_identity(retained.identity()),
            manifest,
        )
    }

    pub fn resolve_candidate(
        retained: &StoreRetainedAuthorityEvidence,
        manifest: &BackupBundleManifest,
        manifest_digest: [u8; 32],
        frontier: ExactRecoveryFrontier,
    ) -> Result<ResolvedRollbackCandidate, RollbackReplayDenial> {
        let retained_authority =
            StoreCurrentAuthorityIdentity::from_aspect_identity(retained.identity());
        if frontier.authority_identity() != retained_authority {
            return Err(RollbackReplayDenial::RetainedAuthorityMismatch);
        }
        if manifest_digest == [0; 32]
            || manifest.root_generation() == 0
            || frontier.wal_structural() != manifest.wal_half_open_interval().1
            || frontier.checkpoint_durability() != manifest.durable_checkpoint_lsn()
            || frontier.client_acknowledged() != manifest.acknowledged_frontier()
        {
            return Err(RollbackReplayDenial::InvalidSource);
        }
        let source_lineage = retained_source_lineage(retained_authority, manifest);
        if source_lineage != frontier.source_lineage() {
            return Err(RollbackReplayDenial::RetainedAuthorityMismatch);
        }
        Ok(ResolvedRollbackCandidate {
            retained_authority,
            source_identity: manifest_digest,
            source_lineage,
            frontier,
            root_generation: manifest.root_generation(),
            manifest_digest,
            wal_start_lsn: manifest.wal_half_open_interval().0,
        })
    }

    pub fn lower(
        candidate: &ResolvedRollbackCandidate,
        staging: &NonCurrentStagingPlanBinding,
    ) -> Result<RollbackReplayPlan, RollbackReplayDenial> {
        let mut digest = Sha256::new();
        digest.update(b"worth-store-rollback-replay-plan-v1");
        digest.update(candidate.source_identity);
        digest.update(candidate.frontier.identity());
        digest.update(candidate.retained_authority.fingerprint());
        digest.update(staging.fingerprint());
        Ok(RollbackReplayPlan {
            fingerprint: digest.finalize().into(),
            staging_plan_fingerprint: staging.fingerprint(),
            frontier: candidate.frontier,
            source_identity: candidate.source_identity,
            retained_authority: candidate.retained_authority,
            manifest_digest: candidate.manifest_digest,
            wal_start_lsn: candidate.wal_start_lsn,
        })
    }

    pub fn execute(
        plan: RollbackReplayPlan,
        staging: NonCurrentStagingMutationScope<'_>,
        application_port: &impl StagedWalApplicationPort,
    ) -> Result<RollbackExecutionReceipt, RollbackReplayDenial> {
        if plan.staging_plan_fingerprint != staging.staging_plan_fingerprint() {
            return Err(RollbackReplayDenial::StagingPlanMismatch);
        }
        let replay_source = validate_staged_wal_replay_source(
            staging,
            plan.staging_plan_fingerprint,
            plan.manifest_digest,
            (plan.wal_start_lsn, plan.frontier.wal_structural()),
        )
        .map_err(RollbackReplayDenial::ReplaySource)?;
        let application = apply_staged_wal(
            staging,
            replay_source,
            plan.frontier.identity(),
            application_port,
        )
        .map_err(RollbackReplayDenial::Application)?;
        Ok(RollbackExecutionReceipt {
            plan_fingerprint: plan.fingerprint,
            frontier: plan.frontier,
            replay_source,
            application,
        })
    }
}

fn retained_source_lineage(
    retained_authority: StoreCurrentAuthorityIdentity,
    manifest: &BackupBundleManifest,
) -> [u8; 32] {
    let mut lineage = Sha256::new();
    lineage.update(b"worth-store-retained-rollback-lineage-v1");
    lineage.update(retained_authority.fingerprint());
    lineage.update(manifest.cut_identity());
    lineage.update(manifest.root_generation().to_be_bytes());
    lineage.finalize().into()
}
