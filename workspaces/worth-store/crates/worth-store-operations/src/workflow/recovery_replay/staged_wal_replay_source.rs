//! Bounded verification of an operation-owned staged WAL source.

use sha2::{Digest, Sha256};
use worth_store_physical_backend::NonCurrentStagingMutationScope;
use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFamily, BackupBundleFormatAuthority,
    BackupBundleFormatDenial, BackupBundleManifestReadLimits,
};
use worth_store_wal::artifact_store::{
    verify_bounded_wal_segment, BoundedWalSegmentDenial, BoundedWalSegmentVerificationRequest,
};

#[derive(Debug)]
pub enum StagedWalReplaySourceDenial {
    StagingPlanMismatch,
    Manifest(BackupBundleFormatDenial),
    ManifestDigestMismatch,
    MissingWal,
    InvalidOwner,
    InvalidCoverage,
    AllocationFailed,
    CounterOverflow,
    Wal(BoundedWalSegmentDenial),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StagedWalReplaySourceReceipt {
    identity: [u8; 32],
    manifest_digest: [u8; 32],
    frame_count: u64,
    bytes_verified: u64,
    interval: (u64, u64),
}

impl StagedWalReplaySourceReceipt {
    pub const fn identity(self) -> [u8; 32] {
        self.identity
    }
    pub const fn manifest_digest(self) -> [u8; 32] {
        self.manifest_digest
    }
    pub const fn frame_count(self) -> u64 {
        self.frame_count
    }
    pub const fn bytes_verified(self) -> u64 {
        self.bytes_verified
    }
    pub const fn interval(self) -> (u64, u64) {
        self.interval
    }
}

pub(crate) fn validate_staged_wal_replay_source(
    media: NonCurrentStagingMutationScope<'_>,
    expected_staging_plan: [u8; 32],
    expected_manifest_digest: [u8; 32],
    expected_interval: (u64, u64),
) -> Result<StagedWalReplaySourceReceipt, StagedWalReplaySourceDenial> {
    if media.staging_plan_fingerprint() != expected_staging_plan {
        return Err(StagedWalReplaySourceDenial::StagingPlanMismatch);
    }
    let materialized = BackupBundleFormatAuthority::canonical()
        .admit_materialized_with_limits(media.root(), BackupBundleManifestReadLimits::canonical())
        .map_err(StagedWalReplaySourceDenial::Manifest)?;
    if materialized.manifest_digest() != expected_manifest_digest {
        return Err(StagedWalReplaySourceDenial::ManifestDigestMismatch);
    }
    let mut rows = Vec::new();
    rows.try_reserve_exact(materialized.manifest().artifacts().len())
        .map_err(|_| StagedWalReplaySourceDenial::AllocationFailed)?;
    rows.extend(
        materialized
            .manifest()
            .artifacts()
            .iter()
            .filter(|row| row.family() == BackupBundleArtifactFamily::WalSegment),
    );
    rows.sort_by_key(|row| match row.coverage() {
        BackupBundleArtifactCoverage::WalSegment { start_lsn, .. } => *start_lsn,
        _ => u64::MAX,
    });
    if rows.is_empty() {
        return Err(StagedWalReplaySourceDenial::MissingWal);
    }
    let mut cursor = expected_interval.0;
    let mut frames = 0_u64;
    let mut bytes = 0_u64;
    let mut identity = Sha256::new();
    identity.update(b"worth-store-staged-wal-replay-source-v1");
    identity.update(expected_staging_plan);
    identity.update(expected_manifest_digest);
    for row in rows {
        let BackupBundleArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } = row.coverage()
        else {
            return Err(StagedWalReplaySourceDenial::InvalidCoverage);
        };
        if *start_lsn != cursor || *end_exclusive_lsn > expected_interval.1 {
            return Err(StagedWalReplaySourceDenial::InvalidCoverage);
        }
        let segment = row
            .reclaim_owner()
            .generation_owner()
            .and_then(|owner| owner.segment_id())
            .ok_or(StagedWalReplaySourceDenial::InvalidOwner)?;
        let request = BoundedWalSegmentVerificationRequest::new(
            segment.get(),
            row.generation(),
            *start_lsn,
            *end_exclusive_lsn,
            row.bytes(),
            row.content_digest(),
            64 * 1024,
        )
        .ok_or(StagedWalReplaySourceDenial::InvalidCoverage)?;
        let observed = verify_bounded_wal_segment(&media.root().join(row.output_name()), request)
            .map_err(StagedWalReplaySourceDenial::Wal)?;
        frames = frames
            .checked_add(observed.frame_count())
            .ok_or(StagedWalReplaySourceDenial::CounterOverflow)?;
        bytes = bytes
            .checked_add(observed.bytes_read())
            .ok_or(StagedWalReplaySourceDenial::CounterOverflow)?;
        identity.update(start_lsn.to_be_bytes());
        identity.update(end_exclusive_lsn.to_be_bytes());
        identity.update(observed.artifact_digest());
        cursor = *end_exclusive_lsn;
    }
    if cursor != expected_interval.1 {
        return Err(StagedWalReplaySourceDenial::InvalidCoverage);
    }
    Ok(StagedWalReplaySourceReceipt {
        identity: identity.finalize().into(),
        manifest_digest: expected_manifest_digest,
        frame_count: frames,
        bytes_verified: bytes,
        interval: expected_interval,
    })
}
