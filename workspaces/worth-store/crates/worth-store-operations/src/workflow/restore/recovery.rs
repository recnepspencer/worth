use sha2::{Digest, Sha256};
use worth_store_physical_backend::{NonCurrentStagingMutationScope, NonCurrentStagingPlanBinding};
use worth_store_physical_format::BackupBundleManifest;

use crate::workflow::recovery_replay::{
    apply_staged_wal, validate_staged_wal_replay_source, StagedWalApplicationDenial,
    StagedWalApplicationPort, StagedWalApplicationReceipt, StagedWalReplaySourceDenial,
    StagedWalReplaySourceReceipt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupRestoreReplayRequest {
    source_cut_identity: [u8; 32],
    source_manifest_digest: [u8; 32],
    target_staging_plan_fingerprint: [u8; 32],
    durable_checkpoint_lsn: u64,
    wal_start_lsn: u64,
    wal_end_exclusive_lsn: u64,
    acknowledged_frontier: u64,
    root_generation: u64,
}

impl BackupRestoreReplayRequest {
    pub fn from_verified_backup(
        manifest: &BackupBundleManifest,
        source_manifest_digest: [u8; 32],
        staging: &NonCurrentStagingPlanBinding,
    ) -> Self {
        let (wal_start_lsn, wal_end_exclusive_lsn) = manifest.wal_half_open_interval();
        Self {
            source_cut_identity: manifest.cut_identity(),
            source_manifest_digest,
            target_staging_plan_fingerprint: staging.fingerprint(),
            durable_checkpoint_lsn: manifest.durable_checkpoint_lsn(),
            wal_start_lsn,
            wal_end_exclusive_lsn,
            acknowledged_frontier: manifest.acknowledged_frontier(),
            root_generation: manifest.root_generation(),
        }
    }
}

#[derive(Debug)]
pub enum BackupRestoreReplayDenial {
    InvalidFrontier,
    StagingPlanMismatch,
    StagingBytesIncomplete,
    ReplaySource(StagedWalReplaySourceDenial),
    Application(StagedWalApplicationDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupRestoreReplayPlan {
    fingerprint: [u8; 32],
    request: BackupRestoreReplayRequest,
}

impl BackupRestoreReplayPlan {
    pub const fn fingerprint(self) -> [u8; 32] {
        self.fingerprint
    }
    pub const fn target_staging_plan_fingerprint(self) -> [u8; 32] {
        self.request.target_staging_plan_fingerprint
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveredBackupFrontierReceipt {
    plan_fingerprint: [u8; 32],
    durable_checkpoint_lsn: u64,
    wal_end_exclusive_lsn: u64,
    acknowledged_frontier: u64,
    root_generation: u64,
    replay_source: StagedWalReplaySourceReceipt,
    application: StagedWalApplicationReceipt,
}

impl RecoveredBackupFrontierReceipt {
    pub const fn plan_fingerprint(self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn durable_checkpoint_lsn(self) -> u64 {
        self.durable_checkpoint_lsn
    }
    pub const fn wal_end_exclusive_lsn(self) -> u64 {
        self.wal_end_exclusive_lsn
    }
    pub const fn acknowledged_frontier(self) -> u64 {
        self.acknowledged_frontier
    }
    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }
    pub const fn replay_source(self) -> StagedWalReplaySourceReceipt {
        self.replay_source
    }
    pub const fn application(self) -> StagedWalApplicationReceipt {
        self.application
    }
}

impl worth_store_physical_backend::NonCurrentStagingOwnerEffect for RecoveredBackupFrontierReceipt {
    fn effect_fingerprint(&self) -> [u8; 32] {
        self.application.identity()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BackupRestoreReplayOwner;

impl BackupRestoreReplayOwner {
    pub fn lower(
        request: BackupRestoreReplayRequest,
    ) -> Result<BackupRestoreReplayPlan, BackupRestoreReplayDenial> {
        if request.wal_start_lsn > request.durable_checkpoint_lsn
            || request.durable_checkpoint_lsn > request.wal_end_exclusive_lsn
            || request.acknowledged_frontier < request.wal_end_exclusive_lsn
            || request.root_generation == 0
        {
            return Err(BackupRestoreReplayDenial::InvalidFrontier);
        }
        let mut digest = Sha256::new();
        digest.update(b"worth-store-backup-restore-replay-plan-v1");
        digest.update(request.source_cut_identity);
        digest.update(request.source_manifest_digest);
        digest.update(request.target_staging_plan_fingerprint);
        digest.update(request.durable_checkpoint_lsn.to_be_bytes());
        digest.update(request.wal_start_lsn.to_be_bytes());
        digest.update(request.wal_end_exclusive_lsn.to_be_bytes());
        digest.update(request.acknowledged_frontier.to_be_bytes());
        digest.update(request.root_generation.to_be_bytes());
        Ok(BackupRestoreReplayPlan {
            fingerprint: digest.finalize().into(),
            request,
        })
    }

    pub fn execute(
        plan: BackupRestoreReplayPlan,
        staging: NonCurrentStagingMutationScope<'_>,
        application_port: &impl StagedWalApplicationPort,
    ) -> Result<RecoveredBackupFrontierReceipt, BackupRestoreReplayDenial> {
        if staging.staging_plan_fingerprint() != plan.request.target_staging_plan_fingerprint {
            return Err(BackupRestoreReplayDenial::StagingPlanMismatch);
        }
        let replay_source = validate_staged_wal_replay_source(
            staging,
            plan.request.target_staging_plan_fingerprint,
            plan.request.source_manifest_digest,
            (
                plan.request.wal_start_lsn,
                plan.request.wal_end_exclusive_lsn,
            ),
        )
        .map_err(BackupRestoreReplayDenial::ReplaySource)?;
        let application = apply_staged_wal(
            staging,
            replay_source,
            backup_frontier_identity(&plan.request),
            application_port,
        )
        .map_err(BackupRestoreReplayDenial::Application)?;
        Ok(RecoveredBackupFrontierReceipt {
            plan_fingerprint: plan.fingerprint,
            durable_checkpoint_lsn: plan.request.durable_checkpoint_lsn,
            wal_end_exclusive_lsn: plan.request.wal_end_exclusive_lsn,
            acknowledged_frontier: plan.request.acknowledged_frontier,
            root_generation: plan.request.root_generation,
            replay_source,
            application,
        })
    }
}

fn backup_frontier_identity(request: &BackupRestoreReplayRequest) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-backup-recovery-frontier-v1");
    digest.update(request.durable_checkpoint_lsn.to_be_bytes());
    digest.update(request.wal_end_exclusive_lsn.to_be_bytes());
    digest.update(request.acknowledged_frontier.to_be_bytes());
    digest.update(request.root_generation.to_be_bytes());
    digest.finalize().into()
}
