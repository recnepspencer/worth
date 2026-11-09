//! Immutable selected members with per-locator reads through the external Native port.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;

mod protected_reads;

#[derive(Debug)]
pub struct ImmutableCheckpointMemberSource {
    pub members: BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    pub commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub canonical_members: BTreeMap<RelationalDurableMemberLocator, PatchStreamPosition>,
    pub canonical_member_reads: std::sync::Mutex<Vec<RelationalDurableMemberLocator>>,
    pub journal_reads: std::sync::Mutex<Vec<PatchStreamPosition>>,
    // These counters retain their record/index/lineage/symbol member meaning.
    pub damaged: AtomicBool,
    pub reads: AtomicUsize,
    pub member_reads: std::sync::Mutex<Vec<RelationalDurableMemberLocator>>,
    pub commit_reads: std::sync::Mutex<Vec<PatchStreamPosition>>,
    pub commits_damaged: AtomicBool,
    pub failed_member: std::sync::Mutex<Option<RelationalDurableMemberLocator>>,
    pub failures: std::sync::Mutex<BTreeMap<ColdRead, Cause>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColdRead {
    Member(RelationalDurableMemberLocator),
    Commit(PatchStreamPosition),
}

impl RelationalDurableColdSource for ImmutableCheckpointMemberSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableMemberCustody,
        protected_reads::Refusal,
    > {
        // The same immutable source owns every addressed byte, so its basis
        // payload has no separate physical pin. Fund the actual custody owner.
        worth_relational::facade::durability::RelationalDurableMemberCustody::try_new_from_source_for_request(
            issuer,
            request,
            || Ok(()),
        )
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational::facade::durability::RelationalDurableMemberRetirementRefusal>
    {
        protected_reads::validate_retirement(original)
    }

    fn advance_member_custody_retirement(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational::facade::durability::RelationalDurableMemberRetirementObservation {
        match protected_reads::validate_retirement(original) {
            Ok(()) => {
                worth_relational::facade::durability::RelationalDurableMemberRetirementObservation::Complete
            }
            Err(cause) => {
                worth_relational::facade::durability::RelationalDurableMemberRetirementObservation::Refused(
                    cause,
                )
            }
        }
    }
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            worth_relational::facade::durability::RelationalDurableReadBounds,
            worth_relational::facade::durability::RelationalDurableReadPayload,
        ),
        protected_reads::Refusal,
    > {
        let original = self.checkpoint_selected_read_protection(request)?;
        Ok((
            worth_relational::facade::durability::RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: self
                    .commits
                    .keys()
                    .next_back()
                    .copied()
                    .unwrap_or(PatchStreamPosition(0)),
                transitions: worth_relational::facade::branch::RelationalBranchTransitionSequence(
                    0,
                ),
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
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableReadPayload,
        protected_reads::Refusal,
    > {
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
    ) -> Result<worth_relational::facade::durability::RelationalDurableMemberCustody, Cause> {
        Ok(worth_relational::facade::durability::RelationalDurableMemberCustody::new(()))
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

impl ImmutableCheckpointMemberSource {
    /// A new physical placement with one actual addressed member absent.
    pub fn without_member(&self, member: RelationalDurableMemberLocator) -> std::sync::Arc<Self> {
        let mut members = self.members.clone();
        assert!(
            members.remove(&member).is_some(),
            "omit an actual producer member"
        );
        self.with_physical_members(members)
    }

    /// Install actual newly emitted members over the original immutable placement.
    pub fn with_published_members(
        &self,
        entries: &[worth_relational::facade::durability::RelationalCheckpointMember],
    ) -> std::sync::Arc<Self> {
        use worth_relational::facade::durability::RelationalCheckpointMember as Member;
        let mut members = self.members.clone();
        for entry in entries {
            match entry {
                Member::New { locator, bytes } => {
                    members.insert(*locator, bytes.clone());
                }
                Member::Carried { locator } => {
                    assert!(
                        members.contains_key(locator),
                        "carried placement has its original bytes"
                    );
                }
            }
        }
        self.with_physical_members(members)
    }

    fn with_physical_members(
        &self,
        members: BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    ) -> std::sync::Arc<Self> {
        let mut canonical_members = self.canonical_members.clone();
        for (locator, bytes) in &members {
            if let Ok(artifact) = worth_relational::facade::durability::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.as_ref()) {
                canonical_members.insert(*locator, artifact.position());
            }
        }
        std::sync::Arc::new(Self {
            members,
            commits: self.commits.clone(),
            canonical_members,
            canonical_member_reads: Default::default(),
            journal_reads: Default::default(),
            damaged: AtomicBool::new(false),
            reads: AtomicUsize::new(0),
            member_reads: Default::default(),
            commit_reads: Default::default(),
            commits_damaged: AtomicBool::new(false),
            failed_member: Default::default(),
            failures: Default::default(),
        })
    }

    fn commit(&self, position: PatchStreamPosition, limit: u64) -> Result<&Vec<u8>, Cause> {
        self.journal_reads.lock().unwrap().push(position);
        self.observe_canonical_read(position)?;
        let bytes = self.commits.get(&position).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes)
    }
    fn observe_canonical_read(&self, position: PatchStreamPosition) -> Result<(), Cause> {
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
        Ok(())
    }
    fn member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<&worth_execution::ExecutionImmutableBytes, Cause> {
        if let Some(position) = self.canonical_members.get(&locator) {
            self.canonical_member_reads.lock().unwrap().push(locator);
            self.observe_canonical_read(*position)?;
        } else {
            self.reads.fetch_add(1, Ordering::Relaxed);
            self.member_reads.lock().unwrap().push(locator);
        }
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
