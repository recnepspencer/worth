use sha2::{Digest, Sha256};
use worth_store_physical_backend::{NonCurrentStagingMutationScope, NonCurrentStagingOwnerEffect};

use super::staged_wal_replay_source::StagedWalReplaySourceReceipt;

#[derive(Debug, Clone, Copy)]
pub struct StagedWalApplicationRequest<'a> {
    staging: NonCurrentStagingMutationScope<'a>,
    replay_source: StagedWalReplaySourceReceipt,
    target_frontier_identity: [u8; 32],
    application_identity: [u8; 32],
}

impl<'a> StagedWalApplicationRequest<'a> {
    pub(crate) fn new(
        staging: NonCurrentStagingMutationScope<'a>,
        replay_source: StagedWalReplaySourceReceipt,
        target_frontier_identity: [u8; 32],
    ) -> Self {
        let application_identity =
            canonical_application_identity(staging, replay_source, target_frontier_identity);
        Self {
            staging,
            replay_source,
            target_frontier_identity,
            application_identity,
        }
    }

    pub const fn staging(self) -> NonCurrentStagingMutationScope<'a> {
        self.staging
    }
    pub const fn replay_source(self) -> StagedWalReplaySourceReceipt {
        self.replay_source
    }
    pub const fn target_frontier_identity(self) -> [u8; 32] {
        self.target_frontier_identity
    }
    /// Stable idempotency identity for the physical WAL application.
    ///
    /// Providers must use this identity for both first execution and recovery
    /// observation. A retry after process loss is the same durable effect, not
    /// permission to mint a second application.
    pub const fn application_identity(self) -> [u8; 32] {
        self.application_identity
    }
}

pub trait StagedWalApplicationPort {
    fn apply_staged_wal(
        &self,
        request: StagedWalApplicationRequest<'_>,
    ) -> Result<StagedWalApplicationProviderReceipt, StagedWalApplicationDenial>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagedWalApplicationProviderReceipt {
    application_identity: [u8; 32],
    staging_plan_fingerprint: [u8; 32],
    replay_source_identity: [u8; 32],
    applied_interval: (u64, u64),
    applied_frames: u64,
    resulting_frontier_identity: [u8; 32],
    durable: bool,
}

impl StagedWalApplicationProviderReceipt {
    #[allow(clippy::too_many_arguments)]
    pub const fn observed(
        application_identity: [u8; 32],
        staging_plan_fingerprint: [u8; 32],
        replay_source_identity: [u8; 32],
        applied_interval: (u64, u64),
        applied_frames: u64,
        resulting_frontier_identity: [u8; 32],
        durable: bool,
    ) -> Self {
        Self {
            application_identity,
            staging_plan_fingerprint,
            replay_source_identity,
            applied_interval,
            applied_frames,
            resulting_frontier_identity,
            durable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagedWalApplicationDenial {
    ProviderUnavailable,
    ProviderRejected,
    ProviderIo(std::io::ErrorKind),
    InvalidTargetFrontier,
    ReceiptMismatch,
    NotDurable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagedWalApplicationReceipt {
    identity: [u8; 32],
    application_identity: [u8; 32],
    replay_source_identity: [u8; 32],
    resulting_frontier_identity: [u8; 32],
    applied_frames: u64,
}

impl StagedWalApplicationReceipt {
    pub const fn identity(self) -> [u8; 32] {
        self.identity
    }
    pub const fn application_identity(self) -> [u8; 32] {
        self.application_identity
    }
    pub const fn replay_source_identity(self) -> [u8; 32] {
        self.replay_source_identity
    }
    pub const fn resulting_frontier_identity(self) -> [u8; 32] {
        self.resulting_frontier_identity
    }
    pub const fn applied_frames(self) -> u64 {
        self.applied_frames
    }
}

impl NonCurrentStagingOwnerEffect for StagedWalApplicationReceipt {
    fn effect_fingerprint(&self) -> [u8; 32] {
        self.identity
    }
}

pub(crate) fn apply_staged_wal(
    staging: NonCurrentStagingMutationScope<'_>,
    replay_source: StagedWalReplaySourceReceipt,
    target_frontier_identity: [u8; 32],
    port: &impl StagedWalApplicationPort,
) -> Result<StagedWalApplicationReceipt, StagedWalApplicationDenial> {
    if target_frontier_identity == [0; 32] {
        return Err(StagedWalApplicationDenial::InvalidTargetFrontier);
    }
    let request =
        StagedWalApplicationRequest::new(staging, replay_source, target_frontier_identity);
    let provider = port.apply_staged_wal(request)?;
    if provider.application_identity != request.application_identity()
        || provider.staging_plan_fingerprint != staging.staging_plan_fingerprint()
        || provider.replay_source_identity != replay_source.identity()
        || provider.applied_interval != replay_source.interval()
        || provider.applied_frames != replay_source.frame_count()
        || provider.resulting_frontier_identity != target_frontier_identity
    {
        return Err(StagedWalApplicationDenial::ReceiptMismatch);
    }
    if !provider.durable {
        return Err(StagedWalApplicationDenial::NotDurable);
    }
    let mut digest = Sha256::new();
    digest.update(b"worth-store-staged-wal-application-v1");
    digest.update(provider.application_identity);
    digest.update(provider.staging_plan_fingerprint);
    digest.update(provider.replay_source_identity);
    digest.update(provider.applied_interval.0.to_be_bytes());
    digest.update(provider.applied_interval.1.to_be_bytes());
    digest.update(provider.applied_frames.to_be_bytes());
    digest.update(provider.resulting_frontier_identity);
    Ok(StagedWalApplicationReceipt {
        identity: digest.finalize().into(),
        application_identity: provider.application_identity,
        replay_source_identity: provider.replay_source_identity,
        resulting_frontier_identity: provider.resulting_frontier_identity,
        applied_frames: provider.applied_frames,
    })
}

fn canonical_application_identity(
    staging: NonCurrentStagingMutationScope<'_>,
    replay_source: StagedWalReplaySourceReceipt,
    target_frontier_identity: [u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-staged-wal-application-command-v1");
    digest.update(staging.staging_plan_fingerprint());
    digest.update(replay_source.identity());
    digest.update(target_frontier_identity);
    digest.finalize().into()
}
