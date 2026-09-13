use worth_store_wal::{BlobWalRecordIdentity, BlobWalRecordKind, CheckpointArtifactObservation};

use crate::{LsmCompactionMembership, LsmMembershipRecord};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_LSM_REPLAY_SOURCE_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LsmReplaySourceIdentity(u64);

impl LsmReplaySourceIdentity {
    fn issue() -> Self {
        let value = NEXT_LSM_REPLAY_SOURCE_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("LSM replay source identity space exhausted");
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LsmReplaySourceKind {
    WalFrame,
    Checkpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LsmReplaySourceDenial {
    MembershipRecordKindMismatch,
    MembershipSequenceNotStrictlyIncreasing,
    MembershipWalGenerationMismatch,
    MembershipWalRangeOverlap,
    CheckpointDoesNotCoverMembership,
    CheckpointDoesNotBindMembership,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedLsmReplaySource {
    identity: LsmReplaySourceIdentity,
    membership: LsmCompactionMembership,
    selected_source: LsmReplaySourceKind,
    selected_first_lsn: u64,
    selected_last_lsn: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LsmReplayExecutionPlan {
    replay_tail: [BlobWalRecordKind; 3],
    replayable_count: u16,
    stale_run_count: u16,
    cleanup_batch_count: u16,
    remaining_run_count: u16,
}

impl AdmittedLsmReplaySource {
    pub fn admit_recovered_membership(
        membership: LsmCompactionMembership,
        checkpoint: Option<&CheckpointArtifactObservation>,
    ) -> Result<Self, LsmReplaySourceDenial> {
        validate_membership(&membership)?;
        let records = membership.record_set();
        let first_wal_lsn = records.value().durable_scope().lsn_start();
        let last_wal_lsn = records.tombstone().durable_scope().lsn_end();
        let (selected_source, selected_first_lsn, selected_last_lsn) = match checkpoint {
            Some(checkpoint) if checkpoint.scope().covered_lsn_end() >= last_wal_lsn => {
                if checkpoint.scope().covered_lsn_start() > first_wal_lsn {
                    return Err(LsmReplaySourceDenial::CheckpointDoesNotCoverMembership);
                }
                if checkpoint.scope().manifest_digest() != membership.replacement_manifest_digest()
                {
                    return Err(LsmReplaySourceDenial::CheckpointDoesNotBindMembership);
                }
                (
                    LsmReplaySourceKind::Checkpoint,
                    checkpoint.scope().covered_lsn_start(),
                    checkpoint.scope().covered_lsn_end(),
                )
            }
            _ => (LsmReplaySourceKind::WalFrame, first_wal_lsn, last_wal_lsn),
        };
        Ok(Self {
            identity: LsmReplaySourceIdentity::issue(),
            membership,
            selected_source,
            selected_first_lsn,
            selected_last_lsn,
        })
    }

    pub const fn identity(&self) -> LsmReplaySourceIdentity {
        self.identity
    }

    pub const fn membership(&self) -> &LsmCompactionMembership {
        &self.membership
    }

    pub const fn selected_source(&self) -> LsmReplaySourceKind {
        self.selected_source
    }

    pub const fn selected_lsn_range(&self) -> (u64, u64) {
        (self.selected_first_lsn, self.selected_last_lsn)
    }

    pub fn identities(&self) -> [BlobWalRecordIdentity; 3] {
        self.membership.identities()
    }

    pub fn execution_plan(&self) -> LsmReplayExecutionPlan {
        let replay_tail = self.identities().map(|identity| identity.kind());
        match self.selected_source {
            LsmReplaySourceKind::WalFrame => LsmReplayExecutionPlan {
                replay_tail,
                replayable_count: replay_tail.len() as u16,
                stale_run_count: 0,
                cleanup_batch_count: 0,
                remaining_run_count: replay_tail.len() as u16
                    + u16::from(self.membership.base().is_some()),
            },
            LsmReplaySourceKind::Checkpoint => LsmReplayExecutionPlan {
                replay_tail,
                replayable_count: 0,
                stale_run_count: replay_tail.len() as u16,
                cleanup_batch_count: 1,
                remaining_run_count: 1,
            },
        }
    }
}

impl LsmReplayExecutionPlan {
    pub const fn replay_tail(self) -> [BlobWalRecordKind; 3] {
        self.replay_tail
    }

    pub const fn replayable_count(self) -> u16 {
        self.replayable_count
    }

    pub const fn stale_run_count(self) -> u16 {
        self.stale_run_count
    }

    pub const fn cleanup_batch_count(self) -> u16 {
        self.cleanup_batch_count
    }

    pub const fn remaining_run_count(self) -> u16 {
        self.remaining_run_count
    }
}

fn validate_membership(membership: &LsmCompactionMembership) -> Result<(), LsmReplaySourceDenial> {
    let records = membership.record_set();
    let value = records.value();
    let generation = records.generation();
    let tombstone = records.tombstone();
    if value.identity().sequence() >= generation.identity().sequence()
        || generation.identity().sequence() >= tombstone.identity().sequence()
    {
        return Err(LsmReplaySourceDenial::MembershipSequenceNotStrictlyIncreasing);
    }
    if !same_wal_generation(value, generation) || !same_wal_generation(generation, tombstone) {
        return Err(LsmReplaySourceDenial::MembershipWalGenerationMismatch);
    }
    if wal_ranges_overlap(value, generation) || wal_ranges_overlap(generation, tombstone) {
        return Err(LsmReplaySourceDenial::MembershipWalRangeOverlap);
    }
    Ok(())
}

fn same_wal_generation(left: &LsmMembershipRecord, right: &LsmMembershipRecord) -> bool {
    left.durable_scope().segment_id() == right.durable_scope().segment_id()
        && left.durable_scope().generation() == right.durable_scope().generation()
}

fn wal_ranges_overlap(left: &LsmMembershipRecord, right: &LsmMembershipRecord) -> bool {
    left.durable_scope().lsn_end() > right.durable_scope().lsn_start()
}
