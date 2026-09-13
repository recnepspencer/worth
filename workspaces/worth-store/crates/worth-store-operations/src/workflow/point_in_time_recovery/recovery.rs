use sha2::{Digest, Sha256};
use worth_store_physical_backend::{NonCurrentStagingMutationScope, NonCurrentStagingPlanBinding};

use crate::workflow::recovery_replay::{
    apply_staged_wal, validate_staged_wal_replay_source, StagedWalApplicationDenial,
    StagedWalApplicationPort, StagedWalApplicationReceipt, StagedWalReplaySourceDenial,
    StagedWalReplaySourceReceipt,
};

use super::ExactRecoveryFrontier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointInTimeReplayRequest {
    exact_frontier: ExactRecoveryFrontier,
    source_identity: [u8; 32],
    staging_plan_fingerprint: [u8; 32],
    staged_manifest_digest: [u8; 32],
    staged_wal_start: u64,
    source_checkpoint_lsn: u64,
    source_wal_end: u64,
    source_acknowledged_frontier: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointInTimeReplaySourceCoordinates {
    pub staged_manifest_digest: [u8; 32],
    pub staged_wal_start: u64,
    pub source_checkpoint_lsn: u64,
    pub source_wal_end: u64,
    pub source_acknowledged_frontier: u64,
}

impl PointInTimeReplayRequest {
    pub fn new(
        exact_frontier: ExactRecoveryFrontier,
        source_identity: [u8; 32],
        staging: &NonCurrentStagingPlanBinding,
        source: PointInTimeReplaySourceCoordinates,
    ) -> Self {
        Self {
            exact_frontier,
            source_identity,
            staging_plan_fingerprint: staging.fingerprint(),
            staged_manifest_digest: source.staged_manifest_digest,
            staged_wal_start: source.staged_wal_start,
            source_checkpoint_lsn: source.source_checkpoint_lsn,
            source_wal_end: source.source_wal_end,
            source_acknowledged_frontier: source.source_acknowledged_frontier,
        }
    }
}

#[derive(Debug)]
pub enum PointInTimeReplayDenial {
    SourceIdentityMismatch,
    TargetBeforeCheckpoint,
    TargetBeyondSource,
    StagingPlanMismatch,
    EmptyStaging,
    ReplaySource(StagedWalReplaySourceDenial),
    Application(StagedWalApplicationDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointInTimeReplayPlan {
    fingerprint: [u8; 32],
    request: PointInTimeReplayRequest,
}

impl PointInTimeReplayPlan {
    pub const fn fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }
    pub const fn exact_frontier(self) -> ExactRecoveryFrontier {
        self.request.exact_frontier
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointInTimeRecoveryReceipt {
    plan_fingerprint: [u8; 32],
    exact_frontier: ExactRecoveryFrontier,
    replay_source: StagedWalReplaySourceReceipt,
    application: StagedWalApplicationReceipt,
}

impl PointInTimeRecoveryReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn exact_frontier(self) -> ExactRecoveryFrontier {
        self.exact_frontier
    }
    pub const fn replay_source(self) -> StagedWalReplaySourceReceipt {
        self.replay_source
    }
    pub const fn application(self) -> StagedWalApplicationReceipt {
        self.application
    }
}

impl worth_store_physical_backend::NonCurrentStagingOwnerEffect for PointInTimeRecoveryReceipt {
    fn effect_fingerprint(&self) -> [u8; 32] {
        self.application.identity()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointInTimeReplayOwner;

impl PointInTimeReplayOwner {
    pub fn lower(
        request: PointInTimeReplayRequest,
    ) -> Result<PointInTimeReplayPlan, PointInTimeReplayDenial> {
        let target = request.exact_frontier;
        if request.source_identity == [0; 32] {
            return Err(PointInTimeReplayDenial::SourceIdentityMismatch);
        }
        if target.checkpoint_durability() < request.source_checkpoint_lsn {
            return Err(PointInTimeReplayDenial::TargetBeforeCheckpoint);
        }
        if target.wal_structural() > request.source_wal_end
            || target.local_durable_commit() > request.source_acknowledged_frontier
            || target.client_acknowledged() > request.source_acknowledged_frontier
        {
            return Err(PointInTimeReplayDenial::TargetBeyondSource);
        }
        let mut digest = Sha256::new();
        digest.update(b"worth-store-pitr-replay-plan-v1");
        digest.update(target.identity());
        digest.update(request.source_identity);
        digest.update(request.staging_plan_fingerprint);
        Ok(PointInTimeReplayPlan {
            fingerprint: digest.finalize().into(),
            request,
        })
    }

    pub fn execute(
        plan: PointInTimeReplayPlan,
        staging: NonCurrentStagingMutationScope<'_>,
        application_port: &impl StagedWalApplicationPort,
    ) -> Result<PointInTimeRecoveryReceipt, PointInTimeReplayDenial> {
        if staging.staging_plan_fingerprint() != plan.request.staging_plan_fingerprint {
            return Err(PointInTimeReplayDenial::StagingPlanMismatch);
        }
        let replay_source = validate_staged_wal_replay_source(
            staging,
            plan.request.staging_plan_fingerprint,
            plan.request.staged_manifest_digest,
            (
                plan.request.staged_wal_start,
                plan.request.exact_frontier.wal_structural(),
            ),
        )
        .map_err(PointInTimeReplayDenial::ReplaySource)?;
        let application = apply_staged_wal(
            staging,
            replay_source,
            plan.request.exact_frontier.identity(),
            application_port,
        )
        .map_err(PointInTimeReplayDenial::Application)?;
        Ok(PointInTimeRecoveryReceipt {
            plan_fingerprint: plan.fingerprint,
            exact_frontier: plan.request.exact_frontier,
            replay_source,
            application,
        })
    }
}
