//! Fault-selectable cold port shared by real checkpoint recovery courts.
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
pub(super) struct MemberSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    pub(super) commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) damaged: AtomicBool,
    pub(super) reads: AtomicUsize,
    pub(super) member_reads: std::sync::Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) commit_reads: std::sync::Mutex<Vec<PatchStreamPosition>>,
    pub(super) commits_damaged: AtomicBool,
    pub(super) failed_member: std::sync::Mutex<Option<RelationalDurableMemberLocator>>,
}

impl RelationalDurableColdSource for MemberSource {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.member_reads.lock().unwrap().push(locator);
        if self.failed_member.lock().unwrap().as_ref() == Some(&locator) {
            return Err(Cause::Damaged);
        }
        let bytes = self.members.get(&locator).ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        let mut bytes = bytes.to_vec();
        if self.damaged.load(Ordering::Relaxed) && bytes.starts_with(b"WORTH-REL-REG") {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.commit_reads.lock().unwrap().push(position);
        if self.commits_damaged.load(Ordering::Relaxed) {
            return Err(Cause::Damaged);
        }
        let bytes = self
            .commits
            .get(&position)
            .ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
}
