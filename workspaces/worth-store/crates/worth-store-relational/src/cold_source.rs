//! Read-only placement adapter. Relational verifies its own member and artifact
//! frames after Store has verified physical integrity and current membership.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberReadFailure, AnchoredRecordName,
    AnchoredRecordReadFailure, OrderedJournalReadFailure, ServingPhysicalRuntime,
};

#[cfg(test)]
mod tests;

/// Calls through the home's demand-read port, including typed refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalStoreColdReadCounters {
    pub member_reads: u64,
    pub commit_reads: u64,
}

pub(crate) struct StoreColdSource {
    serving: Arc<ServingPhysicalRuntime>,
    member_reads: AtomicU64,
    commit_reads: AtomicU64,
}

impl std::fmt::Debug for StoreColdSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreColdSource").finish_non_exhaustive()
    }
}

impl StoreColdSource {
    pub(crate) fn new(serving: Arc<ServingPhysicalRuntime>) -> Self {
        Self {
            serving,
            member_reads: AtomicU64::new(0),
            commit_reads: AtomicU64::new(0),
        }
    }
    pub(crate) fn counters(&self) -> RelationalStoreColdReadCounters {
        RelationalStoreColdReadCounters {
            member_reads: self.member_reads.load(Ordering::Relaxed),
            commit_reads: self.commit_reads.load(Ordering::Relaxed),
        }
    }
}

impl RelationalDurableColdSource for StoreColdSource {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.member_reads.fetch_add(1, Ordering::Relaxed);
        if locator.length > limit {
            return Err(Cause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        let records = self
            .serving
            .anchored_records()
            .map_err(|_| Cause::SourceUnavailable)?;
        let anchor = AnchoredRecordName::new("relational.checkpoint").expect("static anchor");
        let record = records
            .read(anchor, 1 << 30)
            .map_err(|failure| anchor_failure(failure, 1 << 30))?
            .ok_or(Cause::SourceUnavailable)?;
        let placed = record
            .members()
            .iter()
            .find(|member| member.sha256() == locator.sha256 && member.length() == locator.length)
            .copied()
            .ok_or(Cause::SourceUnavailable)?;
        records
            .read_member(anchor, placed, limit)
            .map_err(|failure| match failure {
                AnchoredRecordMemberReadFailure::LimitExceeded { length } => {
                    Cause::LimitExceeded { length, limit }
                }
                AnchoredRecordMemberReadFailure::Damaged(
                    AnchoredRecordMemberDamage::DigestMismatch,
                ) => Cause::IntegrityMismatch,
                AnchoredRecordMemberReadFailure::Damaged(_) => Cause::Damaged,
                AnchoredRecordMemberReadFailure::Value(failure) => anchor_failure(failure, 1 << 30),
                _ => Cause::SourceUnavailable,
            })
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.commit_reads.fetch_add(1, Ordering::Relaxed);
        self.serving
            .canonical_commits()
            .map_err(|_| Cause::SourceUnavailable)?
            .read(position.0, limit)
            .map(|record| record.into_bytes())
            .map_err(|failure| match failure {
                OrderedJournalReadFailure::Damaged => Cause::Damaged,
                OrderedJournalReadFailure::LimitExceeded { payload_bytes } => {
                    Cause::LimitExceeded {
                        length: payload_bytes,
                        limit,
                    }
                }
                _ => Cause::SourceUnavailable,
            })
    }
}

fn anchor_failure(failure: AnchoredRecordReadFailure, limit: u64) -> Cause {
    match failure {
        AnchoredRecordReadFailure::Damaged(_) => Cause::Damaged,
        AnchoredRecordReadFailure::LimitExceeded { payload_bytes } => Cause::LimitExceeded {
            length: payload_bytes,
            limit,
        },
        _ => Cause::SourceUnavailable,
    }
}
