//! Fault-selectable cold port shared by real checkpoint recovery courts.
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[path = "cold_source/protected_reads.rs"]
mod protected_reads;

#[derive(Debug)]
pub(super) struct MemberSource {
    pub(super) members:
        BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    pub(super) commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) damaged: AtomicBool,
    pub(super) reads: AtomicUsize,
    pub(super) member_reads: std::sync::Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) commit_reads: std::sync::Mutex<Vec<PatchStreamPosition>>,
    pub(super) commits_damaged: AtomicBool,
    pub(super) failed_member: std::sync::Mutex<Option<RelationalDurableMemberLocator>>,
    pub(super) failures: std::sync::Mutex<BTreeMap<ColdRead, Cause>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ColdRead {
    Member(RelationalDurableMemberLocator),
    Commit(PatchStreamPosition),
}

impl RelationalDurableColdSource for MemberSource {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            crate::durability::medium::RelationalDurableReadBounds,
            crate::durability::medium::RelationalDurableReadPayload,
        ),
        protected_reads::Refusal,
    > {
        let original = self.checkpoint_selected_read_protection(request)?;
        Ok((
            crate::durability::medium::RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: self
                    .commits
                    .keys()
                    .next_back()
                    .copied()
                    .unwrap_or(PatchStreamPosition(0)),
                transitions: crate::branch::RelationalBranchTransitionSequence(0),
            },
            original,
        ))
    }

    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, protected_reads::Refusal> {
        protected_reads::validate(request, original)?;
        let bytes = self
            .commit(position, limit)
            .map_err(protected_reads::Refusal::Storage)?;
        protected_reads::copy(request, bytes)
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::durability::medium::RelationalDurableReadPayload, protected_reads::Refusal>
    {
        request
            .check_live()
            .map_err(|cause| protected_reads::Refusal::Request(cause.into()))?;
        // The source's member map is immutable for its entire Arc lifetime.
        // Capturing custody reads no member and never selects a newer root.
        Ok(Box::new(()))
    }

    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, protected_reads::Refusal> {
        protected_reads::validate(request, original)?;
        let bytes = self
            .member(locator, limit)
            .map_err(protected_reads::Refusal::Storage)?;
        if self.damaged.load(Ordering::Relaxed) && bytes.starts_with(b"WORTH-REL-REG") {
            protected_reads::damaged_copy(request, bytes)
        } else {
            Ok(bytes.clone())
        }
    }
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let bytes = self.member(locator, limit)?;
        let mut bytes = bytes.to_vec();
        if self.damaged.load(Ordering::Relaxed) && bytes.starts_with(b"WORTH-REL-REG") {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.commit(position, limit).cloned()
    }
}

impl MemberSource {
    fn commit(&self, position: PatchStreamPosition, limit: u64) -> Result<&Vec<u8>, Cause> {
        self.commit_reads.lock().unwrap().push(position);
        if let Some(cause) = self
            .failures
            .lock()
            .unwrap()
            .get(&ColdRead::Commit(position))
        {
            return Err(*cause);
        }
        if self.commits_damaged.load(Ordering::Relaxed) {
            return Err(Cause::Damaged);
        }
        let bytes = self.commits.get(&position).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes)
    }
    fn member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<&worth_execution::ExecutionImmutableBytes, Cause> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.member_reads.lock().unwrap().push(locator);
        if let Some(cause) = self
            .failures
            .lock()
            .unwrap()
            .get(&ColdRead::Member(locator))
        {
            return Err(*cause);
        }
        if self.failed_member.lock().unwrap().as_ref() == Some(&locator) {
            return Err(Cause::Damaged);
        }
        let bytes = self.members.get(&locator).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes)
    }
}
